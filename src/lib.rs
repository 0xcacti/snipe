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
