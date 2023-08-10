use anyhow::Result;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Timelike, Utc};
use chrono_tz::{Tz, TZ_VARIANTS};
use ethers::{
    prelude::{Http, Provider},
    providers::Middleware,
};
use serde::Deserialize;
use serde_json;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub snipe_rpc_url: Option<String>,
    pub time_zone: Option<String>,
}

impl Config {
    pub fn new(snipe_rpc_url: String, time_zone: String) -> Config {
        Config {
            snipe_rpc_url: Some(snipe_rpc_url),
            time_zone: Some(time_zone),
        }
    }

    pub fn snipe_rpc_url(&self) -> &str {
        self.snipe_rpc_url.as_ref().expect("snipe_rpc_url not set")
    }

    pub fn time_zone(&self) -> &str {
        self.time_zone.as_ref().expect("time_zone not set")
    }
}

/// Convert an Ethereum blocknumber to a timestamp
/// if the block number has yet to occur it will return a simple prediction
/// of the timestamp achieved by adding 12 seconds for every block between
/// the current requested block
pub async fn block_to_time(config: Config, block_num: u64) -> Result<String> {
    let tz = parse_timezone(&config.time_zone())?;
    let block_unix = get_block_unix_time(&config, block_num).await?;

    let timestamp = NaiveDateTime::from_timestamp_opt(block_unix as i64, 0).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not convert block unix time to NaiveDateTime: {}",
            block_unix
        )
    })?;

    let utc_datetime: DateTime<Utc> = DateTime::from_utc(timestamp, Utc);
    let datetime = utc_datetime.with_timezone(&tz);
    Ok(datetime.to_string())
}

/// Convert a timestamp in the form YYYY-MM-DD HH:MM:SS
/// to an Ethereum block number.The first block to occur
/// after or at the timestamp will be returned.
/// if the timestamp has yet to occur it will error.
pub async fn time_to_block(config: &Config, time: &str) -> Result<u64> {
    let unix_time = time_to_unix(&config, time)?;
    let block = block_search(&config, unix_time).await?;
    Ok(block)
}

/// log all timezones available
pub fn list_timezones() {
    TZ_VARIANTS.iter().for_each(|tz| println!("{}", tz));
}

// define internal logic

async fn block_search(config: &Config, unix_time: u64) -> Result<u64> {
    let provider = Provider::<Http>::try_from(config.snipe_rpc_url())?;
    let current_block = get_current_block_number(&provider).await?;
    let current_time = get_block_unix_time(&config, current_block).await?;

    if unix_time > current_time {
        return Err(anyhow::anyhow!("Time is in the future."));
    }

    let mut lower_bound = 0;
    let mut upper_bound = current_block;
    let mut current_block = current_block / 2;
    let mut current_time = get_block_unix_time(&config, current_block).await?;

    // write a binary search that finds the first block to occur after or at a given timestamp
    while lower_bound <= upper_bound {
        if current_time == unix_time {
            return Ok(current_block);
        }

        if current_time < unix_time {
            lower_bound = current_block + 1;
        } else {
            upper_bound = current_block - 1;
        }
        current_block = (lower_bound + upper_bound) / 2;
        current_time = get_block_unix_time(&config, current_block).await?;
    }
    if lower_bound > upper_bound {
        current_block = lower_bound;
    }
    Ok(current_block)
}

async fn get_block_unix_time(config: &Config, block_num: u64) -> Result<u64> {
    let provider = Provider::<Http>::try_from(config.snipe_rpc_url())?;
    let current_block = get_current_block_number(&provider).await?;
    if current_block >= block_num {
        return get_block_timestamp(&provider, block_num).await;
    }
    let timestamp = get_block_timestamp(&provider, current_block).await?;
    let time_difference = 12 * (block_num - current_block);
    return Ok(timestamp + time_difference);
}

fn get_genesis() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2015-07-30 15:26:13", "%Y-%m-%d %H:%M:%S").unwrap()
}

fn can_be_genesis(datetime: &Vec<&str>) -> bool {
    let len = datetime.len();

    match len {
        1 => datetime[0] == "2015",
        2 => datetime[0] == "2015" && datetime[1] == "07",
        3 => datetime[0] == "2015" && datetime[1] == "07" && datetime[2] == "30",
        4 => {
            datetime[0] == "2015"
                && datetime[1] == "07"
                && datetime[2] == "30"
                && datetime[3] == "15"
        }
        5 => {
            datetime[0] == "2015"
                && datetime[1] == "07"
                && datetime[2] == "30"
                && datetime[3] == "15"
                && datetime[4] == "26"
        }
        6 => {
            datetime[0] == "2015"
                && datetime[1] == "07"
                && datetime[2] == "30"
                && datetime[3] == "15"
                && datetime[4] == "26"
                && datetime[5] == "13"
        }
        _ => false,
    }
}

fn time_to_unix(config: &Config, time: &str) -> Result<u64> {
    let tz: Tz = config.time_zone().parse().expect("Invalid time zone.");

    let time_components = split_time(time);
    let mut complete_naive_time = vec!["2015", "01", "01", "00", "00", "00"];
    for (i, component) in time_components.iter().enumerate() {
        complete_naive_time[i] = component;
    }

    let date_time_num: Vec<u32> = complete_naive_time
        .iter()
        .map(|x| x.parse::<u32>().unwrap())
        .collect();

    let naive_datetime = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(date_time_num[0] as i32, date_time_num[1], date_time_num[2])
            .unwrap(),
        NaiveTime::from_hms_opt(date_time_num[3], date_time_num[4], date_time_num[5]).unwrap(),
    );

    let aware_datetime = tz
        .with_ymd_and_hms(
            naive_datetime.year(),
            naive_datetime.month(),
            naive_datetime.day(),
            naive_datetime.hour(),
            naive_datetime.minute(),
            naive_datetime.second(),
        )
        .unwrap();

    let utc_datetime = aware_datetime
        .with_timezone(&Utc)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
    let utc_time_components = split_time(utc_datetime.as_str());

    let complete_time_components =
        if can_be_genesis(&utc_time_components[..time_components.len()].to_vec()) {
            vec!["2015", "07", "30", "15", "26", "13"]
        } else {
            let mut full_datetime_parts = vec!["2015", "01", "01", "00", "00", "00"];
            for (i, component) in time_components.iter().enumerate() {
                full_datetime_parts[i] = component;
            }
            full_datetime_parts
        };

    let date_time_num: Vec<u32> = complete_time_components
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

    let utc_datetime: DateTime<Utc> = DateTime::from_utc(datetime, Utc);
    Ok(utc_datetime.timestamp() as u64)
}

fn split_time(time: &str) -> Vec<&str> {
    let parts: Vec<&str> = time.split(&['-', ' ', ':'][..]).collect();
    parts
}

fn parse_timezone(time_zone: &str) -> Result<Tz> {
    let tz: Tz = time_zone.parse().expect("Invalid time zone.");

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

// testing

#[cfg(test)]
mod tests {
    use super::*;

    use dotenv::dotenv;
    use dotenv_codegen::dotenv;

    // helpers
    fn get_test_config() -> Config {
        dotenv().ok();
        let snipe_rpc_url = dotenv!("RPC_URL");
        Config::new(snipe_rpc_url.to_string(), "UTC".to_string())
    }

    fn get_genesis_unix() -> u64 {
        1438269973
    }

    // stub of config
    #[tokio::test]
    async fn historical_block_to_time() {
        let known_time = 1438269988;
        let calculated_time = get_block_unix_time(&get_test_config(), 1).await.unwrap();
        assert_eq!(known_time, calculated_time);
    }

    #[tokio::test]
    async fn future_block_to_time() {
        dotenv().ok();
        let config = get_test_config();
        let provider = Provider::<Http>::try_from(config.snipe_rpc_url()).unwrap();

        let current_block = get_current_block_number(&provider).await.unwrap();
        let current_time = get_block_timestamp(&provider, current_block).await.unwrap();
        let block_num = current_block + 2;
        let estimated_time = get_block_unix_time(&config, block_num).await.unwrap();
        assert!(estimated_time > current_time);
    }

    #[test]
    fn time_zone_to_tz() {
        let tz = parse_timezone("UTC").unwrap();
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
    fn genesis_conversion() {
        // year
        let time = vec!["2016"];
        assert!(!can_be_genesis(&time));

        let time = vec!["2015"];
        assert!(can_be_genesis(&time));

        // year and month
        let time = vec!["2015", "08"];
        assert!(!can_be_genesis(&time));

        let time = vec!["2015", "07"];
        assert!(can_be_genesis(&time));

        // year, month, and day
        let time = vec!["2015", "07", "31"];
        assert!(!can_be_genesis(&time));

        let time = vec!["2015", "07", "30"];
        assert!(can_be_genesis(&time));

        // year, month, day, hour
        let time = vec!["2015", "07", "30", "16"];
        assert!(!can_be_genesis(&time));

        let time = vec!["2015", "07", "30", "15"];
        assert!(can_be_genesis(&time));

        // year, month, day, hour, minute
        let time = vec!["2015", "07", "30", "15", "27"];
        assert!(!can_be_genesis(&time));

        let time = vec!["2015", "07", "30", "15", "26"];
        assert!(can_be_genesis(&time));

        // year, month, day, hour, minute, second
        let time = vec!["2015", "07", "30", "15", "26", "14"];
        assert!(!can_be_genesis(&time));

        let time = vec!["2015", "07", "30", "15", "26", "13"];
        assert!(can_be_genesis(&time));
    }

    #[test]
    fn year_to_unix_genesis() {
        let time = "2015";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, get_genesis_unix());
    }

    #[test]
    fn year_to_unix_first_timestamp() {
        let time = "2016";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1451606400);
    }
    #[test]
    fn month_to_unix_genesis() {
        let time = "2015-07";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, get_genesis_unix());
    }

    #[test]
    fn month_to_unix_first_timestamp() {
        let time = "2016-02";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1454284800);
    }

    #[test]
    fn day_to_unix_genesis() {
        let time = "2015-07-30";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, get_genesis_unix());
    }

    #[test]
    fn hour_to_unix_first_timestamp() {
        let time = "2016-02-01 01";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1454288400);
    }

    #[test]
    fn minute_to_unix_genesis() {
        let time = "2015-07-30 15:26";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, get_genesis_unix());
    }

    #[test]
    fn minute_to_unix_first_timestamp() {
        let time = "2016-02-01 01:07";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1454288820);
    }

    #[test]
    fn second_to_unix_genesis() {
        let time = "2015-07-30 15:26:13";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, get_genesis_unix());
    }

    #[test]
    fn second_to_unix_first_timestamp() {
        let time = "2016-02-01 01:07:05";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1454288825);
    }

    #[test]
    fn tz_time_to_unix_genesis() {
        let mut config = get_test_config();
        config.time_zone = Some("America/New_York".to_string());
        let time = "2015-07-30 11:26:13";
        let unix = time_to_unix(&config, &time).unwrap();
        assert_eq!(unix, get_genesis_unix());
    }

    #[tokio::test]
    async fn block_vague_search() {
        let expected_block = 17816434;
        let unix_time = 1690848000;
        let config = get_test_config();
        let predicted_block = block_search(&config, unix_time).await.unwrap();
        assert_eq!(expected_block, predicted_block);
    }
    #[tokio::test]
    async fn exact_block_search() {
        let expected_block = 17816434;
        let unix_time = 1690848011;
        let config = get_test_config();
        let predicted_block = block_search(&config, unix_time).await.unwrap();
        assert_eq!(expected_block, predicted_block);
    }
}
