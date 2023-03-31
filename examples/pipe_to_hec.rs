use std::io;

/// Pipe stdin to HTTP Event Collector!
use clap::*;
use serde_json::json;
use splunk::get_serverconfig;
use splunk::hec::HecClient;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    index: Option<String>,
    #[arg(short='n', long)]
    hostname: Option<String>,
    #[arg(short, long)]
    port: Option<u16>,
    #[arg(short, long)]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), String> {

    let cli = Cli::parse();

    // in case they're using environment variables
    let serverconfig = get_serverconfig(splunk::ServerConfigType::Hec)?;


    // set up the HecClient
    let mut hec = HecClient::with_serverconfig(serverconfig);

    if let Some(port) = cli.port {
        hec.serverconfig = hec.serverconfig.with_port(port);
    }
    if let Some(hostname) = cli.hostname {
        hec.serverconfig = hec.serverconfig.with_hostname(hostname);
    }

    if let Some(cli_token) = cli.token {
        hec.serverconfig = hec.serverconfig.with_token(cli_token);
    }

    eprintln!("config: {hec:?}");
    eprintln!("Waiting for input...");
    let mut buffer = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    while  stdin.read_line(&mut buffer).map_err(|err| err.to_string())? > 0 {
        if buffer.trim().len() != 0 {
            let data = json!(buffer.trim());
            eprintln!("Sending {data:?}");
            if let Err(err) = hec.send_to_splunk(data).await {
                eprintln!("failed to send: {err:?}");
            };
        }

        buffer.clear();
    }
    Ok(())
}
