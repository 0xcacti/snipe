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
    let timestamp = get_block_timestamp(&provider, current_block).await?;
    if current_block >= block_num {
        return Ok(timestamp);
    }
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

fn time_to_unix_time(time: &str, time_zone: &str) -> Result<u64> {
    let tz: Tz = time_zone.parse().expect("Invalid time zone.");
    println!("time: {}", tz);

    let parsed_datetime: DateTime<Tz> = match time.len() {
        4 => {
            let year: i32 = time
                .parse()
                .context("failed to parse year into valid number")?;
            if year < 2015 {
                return Err(anyhow::anyhow!("year predates Ethereum"));
            }
            let d = NaiveDate::from_ymd_opt(year, 1, 1).unwrap();
            let t = NaiveTime::from_hms_opt(0, 0, 0).unwrap(); // Start of the day
            let dt = NaiveDateTime::new(d, t);
            tz.from_local_datetime(&dt).single().unwrap()
        }
        7 => {
            let year: i32 = time[0..4]
                .parse()
                .context("failed to parse year into valid number")?;
            let month: u32 = time[5..7]
                .parse()
                .context("failed to parse month into valid number")?;
            if year < 2015 {
                return Err(anyhow::anyhow!("year predates Ethereum"));
            }
            let d = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
            let t = NaiveTime::from_hms_opt(0, 0, 0).unwrap(); // Start of the day
            let dt = NaiveDateTime::new(d, t);
            tz.from_local_datetime(&dt).single().unwrap()
        }
        10 => {
            let year: i32 = time[0..4]
                .parse()
                .context("failed to parse year into valid number")?;
            let month: u32 = time[5..7]
                .parse()
                .context("failed to parse month into valid number")?;
            let day: u32 = time[8..10]
                .parse()
                .context("failed to parse day into valid number")?;
            if year < 2015 {
                return Err(anyhow::anyhow!("year predates Ethereum"));
            }
            let d = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            let t = NaiveTime::from_hms_opt(0, 0, 0).unwrap(); // Start of the day
            let dt = NaiveDateTime::new(d, t);
            tz.from_local_datetime(&dt).single().unwrap()
        }
        13 => {
            let year: i32 = time[0..4]
                .parse()
                .context("failed to parse year into valid number")?;
            let month: u32 = time[5..7]
                .parse()
                .context("failed to parse month into valid number")?;
            let day: u32 = time[8..10]
                .parse()
                .context("failed to parse day into valid number")?;
            let hour: u32 = time[11..13]
                .parse()
                .context("failed to parse hour into valid number")?;
            if year < 2015 {
                return Err(anyhow::anyhow!("year predates Ethereum"));
            }
            let d = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            let t = NaiveTime::from_hms_opt(hour, 0, 0).unwrap(); // Start of the hour
            let dt = NaiveDateTime::new(d, t);
            tz.from_local_datetime(&dt).single().unwrap()
        }
        16 => {
            let year: i32 = time[0..4]
                .parse()
                .context("failed to parse year into valid number")?;
            let month: u32 = time[5..7]
                .parse()
                .context("failed to parse month into valid number")?;
            let day: u32 = time[8..10]
                .parse()
                .context("failed to parse day into valid number")?;
            let hour: u32 = time[11..13]
                .parse()
                .context("failed to parse hour into valid number")?;
            let minute: u32 = time[14..16]
                .parse()
                .context("failed to parse minute into valid number")?;
            if year < 2015 {
                return Err(anyhow::anyhow!("year predates Ethereum"));
            }
            let d = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            let t = NaiveTime::from_hms_opt(hour, minute, 0).unwrap(); // Start of the minute
            let dt = NaiveDateTime::new(d, t);
            tz.from_local_datetime(&dt).single().unwrap()
        }
        19 => {
            let year: i32 = time[0..4]
                .parse()
                .context("failed to parse year into valid number")?;
            let month: u32 = time[5..7]
                .parse()
                .context("failed to parse month into valid number")?;
            let day: u32 = time[8..10]
                .parse()
                .context("failed to parse day into valid number")?;
            let hour: u32 = time[11..13]
                .parse()
                .context("failed to parse hour into valid number")?;
            let minute: u32 = time[14..16]
                .parse()
                .context("failed to parse minute into valid number")?;
            let second: u32 = time[17..19]
                .parse()
                .context("failed to parse second into valid number")?;
            if year < 2015 {
                return Err(anyhow::anyhow!("year predates Ethereum"));
            }
            let d = NaiveDate::from_ymd_opt(year, month, day).unwrap();
            let t = NaiveTime::from_hms_opt(hour, minute, second).unwrap();
            let dt = NaiveDateTime::new(d, t);
            tz.from_local_datetime(&dt).single().unwrap()
        }
        _ => {
            return Err(anyhow::anyhow!("invalid timestamp format"));
        }
    };
    Ok(parsed_datetime.timestamp() as u64)
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
