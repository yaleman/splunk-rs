use std::io::Read;

/// Pipe stdin to HTTP Event Collector!
use clap::*;
use serde_json::json;
use splunk::hec::HecClient;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    index: Option<String>,
    #[arg(short = 'n', long)]
    hostname: Option<String>,
    #[arg(short, long)]
    port: Option<u16>,
    #[arg(short, long)]
    token: Option<String>,
    #[arg(short, long)]
    source: Option<String>,
    #[arg(short = 'S', long)]
    sourcetype: Option<String>,
    #[arg(long)]
    no_verify_tls: Option<bool>,
    filename: String,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let cli = Cli::parse();

    let no_verify_tls = match cli.no_verify_tls {
        Some(val) => val,
        None => false,
    };

    // in case they're using environment variables
    // let serverconfig = splunk::ServerConfig::try_from_env(splunk::ServerConfigType::Hec)?;
    let serverconfig = splunk::ServerConfig::default().with_verify_tls(!no_verify_tls);

    // set up the HecClient
    let mut hec = HecClient::with_serverconfig(serverconfig);

    if let Some(port) = cli.port {
        hec.serverconfig = hec.serverconfig.with_port(port);
    }
    if let Some(hostname) = cli.hostname {
        hec.serverconfig = hec.serverconfig.with_hostname(hostname);
    }
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

    // open the file and read the contents into a buffer
    let mut buffer = String::new();
    let mut file = std::fs::File::open(cli.filename).map_err(|err| err.to_string())?;
    file.read_to_string(&mut buffer)
        .map_err(|err| err.to_string())?;

    let data = json!(buffer.trim());
    hec.enqueue(data).await;
    match hec.flush(None).await {
        Ok(val) => eprintln!("Sent {} events!", val),
        Err(err) => eprintln!("Failure sending event: {err:?}"),
    }
    Ok(())
}
