use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rpc_url: Option<String>,
    pub time_zone: Option<String>,
}

impl Config {
    pub fn new(rpc_url: Option<String>, time_zone: Option<String>) -> Config {
        Config { rpc_url, time_zone }
    }

    pub fn rpc_url(&self) -> &str {
        self.rpc_url.as_ref().expect("api_key not set")
    }

    pub fn time_zone(&self) -> &str {
        self.time_zone.as_ref().expect("bible_version not set")
    }
}

pub fn block_to_time(config: Config, block_num: u64) -> String {
    println!("block_to_time");
    "2021-01-01 00:00:00".to_string()
}

pub fn time_to_block(config: Config, time: &str) -> u64 {
    println!("block_to_time");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    // stub of config
    #[test]
    fn test_historical_block_to_time() {
        let config = Config::new(
            Some("https://mainnet.infura.io/v3/".to_string()),
            Some("UTC".to_string()),
        );
        assert_eq!(block_to_time(config, 1), "2021-01-01 00:00:00");
    }
    #[test]
    fn test_future_block_to_time() {}
}
