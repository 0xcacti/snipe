use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use chrono_tz::{Tz, TZ_VARIANTS};
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
        self.rpc_url.as_ref().expect("rpc_url not set")
    }

    pub fn time_zone(&self) -> &str {
        self.time_zone.as_ref().expect("time_zone not set")
    }
}

pub async fn block_to_time(config: Config, block_num: u64) -> Result<u64> {
    // get current block
    let provider = Provider::<Http>::try_from(config.rpc_url())?;
    let current_block = get_current_block_number(&provider).await?;
    if current_block >= block_num {
        return get_block_timestamp(&provider, block_num).await;
    }
    let timestamp = get_block_timestamp(&provider, current_block).await?;
    let time_difference = 12 * (block_num - current_block);
    return Ok(timestamp + time_difference);
}

pub fn time_to_block(config: Config, time: &str) -> Result<u64> {
    let unix_time = time_to_unix_time(time, config.time_zone())?;
    Ok(1)
}

pub fn list_timezones() {
    TZ_VARIANTS.iter().for_each(|tz| println!("{}", tz));
}

fn get_genesis() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2015-07-30 03:26:13", "%Y-%m-%d %H:%M:%S%").unwrap()
}

fn time_to_unix_time(time: &str, time_zone: &str) -> Result<u64> {
    let tz: Tz = time_zone.parse().expect("Invalid time zone.");

    let time_components = split_time(time);
    let mut full_datetime_parts = vec!["00"; 6];
    for (i, component) in time_components.iter().enumerate() {
        full_datetime_parts[i] = component;
    }
    let date_time_num: Vec<u32> = full_datetime_parts
        .iter()
        .map(|x| x.parse::<u32>().unwrap())
        .collect();
    let datetime = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(date_time_num[0] as i32, date_time_num[1], date_time_num[2])
            .unwrap(),
        NaiveTime::from_hms_opt(date_time_num[3], date_time_num[4], date_time_num[5]).unwrap(),
    );

    if datetime < get_genesis() {
        return Err(anyhow::anyhow!("year predates Ethereum"));
    }

    let timestamp = tz.from_local_datetime(&datetime).unwrap();
    Ok(timestamp.timestamp() as u64)
}

fn split_time(time: &str) -> Vec<&str> {
    let major_components = time.split(" ").collect::<Vec<&str>>();
    let mut time_components: Vec<&str> = Vec::new();
    time_components.extend(major_components[0].split("-").collect::<Vec<&str>>());
    if major_components.len() == 2 {
        let minor_components = major_components[1].split(":").collect::<Vec<&str>>();
        time_components.extend(minor_components);
    }
    time_components
}

fn parse_timezone(time_zone: &str) -> Result<Tz> {
    let tz: Tz = time_zone.parse().expect("Invalid time zone.");

    println!("time: {}", tz);

    Ok(tz)
}

async fn get_current_block_number(provider: &Provider<Http>) -> Result<u64> {
    let block_number = provider.get_block_number().await?;
    Ok(block_number.as_u64())
}

async fn get_block_timestamp(provider: &Provider<Http>, block_num: u64) -> Result<u64> {
    let response = provider.get_block(block_num).await?;
    let block_json = serde_json::to_string(&response)?;
    let block: serde_json::Value = serde_json::from_str(&block_json)?;
    let timestamp = block["timestamp"].as_str().unwrap();
    let timestamp_hex = &timestamp[2..];
    let timestamp = u64::from_str_radix(timestamp_hex, 16)?;

    Ok(timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    use dotenv::dotenv;
    use dotenv_codegen::dotenv;
    // stub of config
    #[tokio::test]
    async fn historical_block_to_time() {
        dotenv().ok();
        let rpc_url = dotenv!("RPC_URL");
        let config = Config::new(Some(rpc_url.to_string()), Some("UTC".to_string()));

        let known_time = 1438269988;
        let calculated_time = block_to_time(config, 1).await.unwrap();
        assert_eq!(known_time, calculated_time);
    }

    #[tokio::test]
    async fn future_block_to_time() {
        dotenv().ok();
        let rpc_url = dotenv!("RPC_URL");
        let config = Config::new(Some(rpc_url.to_string()), Some("UTC".to_string()));
        let provider = Provider::<Http>::try_from(config.rpc_url()).unwrap();

        let current_block = get_current_block_number(&provider).await.unwrap();
        let current_time = get_block_timestamp(&provider, current_block).await.unwrap();
        let block_num = current_block + 2;
        let estimated_time = block_to_time(config, block_num).await.unwrap();
        assert!(estimated_time > current_time);
    }

    #[test]
    fn time_zone_to_tz() {
        let tz = parse_timezone("UTC").unwrap();
        println!("tz: {}", tz);
        assert_eq!(tz.name(), "UTC");
    }

    #[test]
    fn split_time_to_components() {
        let time = "2015";
        let time_components = split_time(time);
        assert_eq!(time_components[0], "2015");

        let time = "2015-07";
        let time_components = split_time(time);
        assert_eq!(time_components[0], "2015");
        assert_eq!(time_components[1], "07");

        let time = "2015-07-30";
        let time_components = split_time(time);
        assert_eq!(time_components[0], "2015");
        assert_eq!(time_components[1], "07");
        assert_eq!(time_components[2], "30");

        let time = "2015-07-30 03";
        let time_components = split_time(time);
        assert_eq!(time_components[0], "2015");
        assert_eq!(time_components[1], "07");
        assert_eq!(time_components[2], "30");
        assert_eq!(time_components[3], "03");

        let time = "2015-07-30 03:26";
        let time_components = split_time(time);
        assert_eq!(time_components[0], "2015");
        assert_eq!(time_components[1], "07");
        assert_eq!(time_components[2], "30");
        assert_eq!(time_components[3], "03");
        assert_eq!(time_components[4], "26");

        let time = "2015-07-30 03:26:13";
        let time_components = split_time(time);
        assert_eq!(time_components[0], "2015");
        assert_eq!(time_components[1], "07");
        assert_eq!(time_components[2], "30");
        assert_eq!(time_components[3], "03");
        assert_eq!(time_components[4], "26");
        assert_eq!(time_components[5], "13");
    }

    #[test]
    fn year_to_unix() {}

    #[test]
    fn month_to_unix() {}

    #[test]
    fn day_to_unix() {}

    #[test]
    fn hour_to_unix() {}

    #[test]
    fn minute_to_unix() {}

    #[test]
    fn second_to_unix() {}
}
