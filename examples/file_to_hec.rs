use std::io::Read;

/// Pipe stdin to HTTP Event Collector!
use clap::*;
use serde_json::json;
use splunk::errors::SplunkError;
use splunk::hec::HecClient;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, env = "SPLUNK_INDEX")]
    index: Option<String>,
    #[arg(short = 'n', long, env = "SPLUNK_HOSTNAME")]
    hostname: Option<String>,
    #[arg(short, long, env = "SPLUNK_PORT")]
    port: Option<u16>,
    #[arg(short, long, env = "SPLUNK_TOKEN")]
    token: Option<String>,
    #[arg(short, long, env = "SPLUNK_SOURCE")]
    source: Option<String>,
    #[arg(short = 'S', long, env = "SPLUNK_SOURCETYPE")]
    sourcetype: Option<String>,
    #[arg(long, action, env = "SPLUNK_NO_VERIFY_TLS", default_value_t = false)]
    no_verify_tls: bool,

    filename: String,
}

#[tokio::main]
async fn main() -> Result<(), SplunkError> {
    let cli = Cli::parse();

    // in case they're using environment variables
    let mut serverconfig = splunk::ServerConfig::default().with_verify_tls(!cli.no_verify_tls);
    serverconfig = match cli.token {
        Some(token) => serverconfig.with_token(token),
        None => serverconfig,
    };

    serverconfig = match cli.port {
        Some(port) => serverconfig.with_port(port),
        None => serverconfig.with_port(8088),
    };

    serverconfig = match cli.hostname {
        Some(hostname) => serverconfig.with_hostname(hostname),
        None => serverconfig,
    };

    // set up the HecClient
    let mut hec = HecClient::with_serverconfig(serverconfig);

    if let Some(index) = cli.index {
        hec = hec.with_index(index);
    }
    if let Some(val) = cli.source {
        hec = hec.with_source(val)
    } else {
        hec = hec.with_source(&cli.filename);
    };
    if let Some(val) = cli.sourcetype {
        hec = hec.with_sourcetype(val)
    };

    println!("{:?}", hec);

    // open the file and read the contents into a buffer
    let mut buffer = String::new();
    let mut file =
        std::fs::File::open(cli.filename).map_err(|err| SplunkError::Generic(err.to_string()))?;
    file.read_to_string(&mut buffer)
        .map_err(|err| SplunkError::Generic(err.to_string()))?;

    let data = json!(buffer.trim());
    hec.enqueue(data).await;
    match hec.flush(None).await {
        Ok(val) => eprintln!("Sent {} events!", val),
        Err(err) => eprintln!("Failure sending event: {err:?}"),
    }
    Ok(())
}
