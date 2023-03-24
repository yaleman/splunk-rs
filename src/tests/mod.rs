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

    Ok(ServerConfig {
        hostname,
        port,
        auth_method: crate::search::AuthenticationMethod::Basic {
            username: env::var("SPLUNK_USERNAME").unwrap(),
            password: env::var("SPLUNK_PASSWORD").unwrap(),
        },
        ..Default::default()
    })
}
