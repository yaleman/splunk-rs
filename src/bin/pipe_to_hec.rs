use std::io;

/// Pipe stdin to HTTP Event Collector!
use clap::*;
use serde_json::json;
use splunk::errors::SplunkError;
use splunk::hec::HecClient;

#[derive(Parser)]
#[command(version, about)]
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
    /// Enable debug mode
    #[arg(short, long, action = clap::ArgAction::SetTrue, env)]
    debug: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<(), SplunkError> {
    let cli = Cli::parse();

    // in case they're using environment variables
    let serverconfig = splunk::ServerConfig::try_from_env(splunk::ServerConfigType::Hec)?;

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
    };
    if let Some(val) = cli.sourcetype {
        hec = hec.with_sourcetype(val)
    };

    if cli.debug.unwrap_or_default() {
        eprintln!("config: {hec:?}");
        eprintln!("Waiting for input...");
    }

    let mut buffer = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    while stdin
        .read_line(&mut buffer)
        .map_err(|err| SplunkError::Generic(err.to_string()))?
        > 0
    {
        if buffer.trim().len() != 0 {
            let data = json!(buffer.trim());

            if cli.debug.unwrap_or_default() {
                eprintln!("Sending {data:?}");
            }
            hec.enqueue(data).await;
        }
        if hec.queue_size().await >= 10 {
            match hec.flush(None).await {
                Ok(val) => eprintln!("Sent {} events!", val),
                Err(err) => eprintln!("Failure sending event: {err:?}"),
            }
        }

        buffer.clear();
    }
    match hec.flush(None).await {
        Ok(val) => eprintln!("Sent {} events!", val),
        Err(err) => eprintln!("Failure sending event: {err:?}"),
    }

    Ok(())
}
