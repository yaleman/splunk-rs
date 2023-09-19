use std::io;

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
}

#[tokio::main]
async fn main() -> Result<(), String> {
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

    eprintln!("config: {hec:?}");
    eprintln!("Waiting for input...");
    let mut buffer = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    while stdin
        .read_line(&mut buffer)
        .map_err(|err| err.to_string())?
        > 0
    {
        if buffer.trim().len() != 0 {
            let data = json!(buffer.trim());
            eprintln!("Sending {data:?}");
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
