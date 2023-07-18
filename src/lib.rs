use anyhow::Result;
use ethers::{
    prelude::{Http, Provider},
    providers::Middleware,
};
use serde::Deserialize;
use serde_json;

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
pub async fn block_to_time(config: Config, block_num: u64) -> Result<String> {
    println!("block_to_time");
    println!("{}", block_num);
    let provider = Provider::<Http>::try_from(config.rpc_url())?;
    let response = provider.get_block(block_num).await?;
    let block_json = serde_json::to_string(&response)?;
    let block: serde_json::Value = serde_json::from_str(&block_json)?;
    let timestamp = block["timestamp"].as_str().unwrap();
    let timestamp_hex = &timestamp[2..];
    let timestamp = u64::from_str_radix(timestamp_hex, 16)?;
    Ok(timestamp.to_string())
}

pub fn time_to_block(config: Config, time: &str) -> u64 {
    println!("block_to_time");
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    use dotenv::dotenv;
    use dotenv_codegen::dotenv;
    // stub of config
    #[tokio::test]
    async fn test_historical_block_to_time() {
        dotenv().ok();
        let rpc_url = dotenv!("RPC_URL");
        let config = Config::new(Some(rpc_url.to_string()), Some("UTC".to_string()));

        let known_time = 1438269988.to_string();
        let calculated_time = block_to_time(config, 1).await.unwrap();
        assert_eq!(known_time, calculated_time);
    }
    #[test]
    fn test_future_block_to_time() {}
}
