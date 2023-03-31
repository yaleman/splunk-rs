use std::env;

use crate::ServerConfig;

mod hec;

mod search;

#[allow(dead_code)]
pub enum TestServerConfig {
    Hec,
    Api,
}

#[allow(dead_code)]
pub fn get_serverconfig(configtype: TestServerConfig) -> Result<ServerConfig, String> {
    let env_prefix = match configtype {
        TestServerConfig::Hec => "SPLUNK_HEC_",
        TestServerConfig::Api => "SPLUNK_API_",
    };

    let hostname = match env::var(format!("{env_prefix}HOSTNAME")) {
        Ok(val) => val,
        Err(_) => {
            let error = format!("Please ensure env var {env_prefix}HOSTNAME is set");
            eprintln!("{}", error);
            return Err(error);
        }
    };
    let port = match env::var(format!("{env_prefix}PORT")) {
        Ok(val) => val,
        Err(_) => 8089.to_string(),
    };
    let port: u16 = port.parse::<u16>().unwrap();

    let config = ServerConfig::new(hostname).with_port(port);
    let config = match configtype {
        TestServerConfig::Hec => {
            let token = env::var(format!("{env_prefix}TOKEN"))
                .expect("Couldn't get SPLUNK_HEC_TOKEN env var");
            config.with_token(token)
        }
        TestServerConfig::Api => config.with_username_password(
            env::var("SPLUNK_USERNAME").expect("Couldn't get SPLUNK_USERNAME env var!"),
            env::var("SPLUNK_PASSWORD").expect("Couldn't get SPLUNK_PASSWORD env var!"),
        ),
    };
    Ok(config)
}
