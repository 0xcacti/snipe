use anyhow::{Context, Result};
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
    pub rpc_url: Option<String>,
    pub time_zone: Option<String>,
    pub format: Option<String>,
}

impl Config {
    pub fn new(rpc_url: String, time_zone: String, format: String) -> Config {
        Config {
            rpc_url: Some(rpc_url),
            time_zone: Some(time_zone),
            format: Some(format),
        }
    }

    pub fn rpc_url(&self) -> &str {
        self.rpc_url.as_ref().expect("rpc_url not set")
    }

    pub fn time_zone(&self) -> &str {
        self.time_zone.as_ref().expect("time_zone not set")
    }

    pub fn format(&self) -> &str {
        self.format.as_ref().expect("format not set")
    }
}

pub async fn block_to_time(config: Config, block_num: u64) -> Result<String> {
    let tz = parse_timezone(&config.time_zone())?;
    let block_unix = get_block_unix_time(&config, block_num).await?;
    let timestamp = NaiveDateTime::from_timestamp_opt(block_unix as i64, 0).unwrap();
    let utc_datetime: DateTime<Utc> = DateTime::from_utc(timestamp, Utc);
    let datetime = utc_datetime.with_timezone(&tz);
    let datetime = datetime.format(&config.format()).to_string();
    Ok(datetime)
}

pub fn time_to_block(config: &Config, time: &str) -> Result<u64> {
    let unix_time = time_to_unix(&config, time)?;

    Ok(1)
}

async fn get_block_unix_time(config: &Config, block_num: u64) -> Result<u64> {
    let provider = Provider::<Http>::try_from(config.rpc_url())?;
    let current_block = get_current_block_number(&provider).await?;
    if current_block >= block_num {
        return get_block_timestamp(&provider, block_num).await;
    }
    let timestamp = get_block_timestamp(&provider, current_block).await?;
    let time_difference = 12 * (block_num - current_block);
    return Ok(timestamp + time_difference);
}

pub fn list_timezones() {
    TZ_VARIANTS.iter().for_each(|tz| println!("{}", tz));
}

fn get_genesis() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2015-07-30 03:26:13", "%Y-%m-%d %H:%M:%S").unwrap()
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
                && datetime[3] == "03"
        }
        5 => {
            datetime[0] == "2015"
                && datetime[1] == "07"
                && datetime[2] == "30"
                && datetime[3] == "03"
                && datetime[4] == "26"
        }
        6 => {
            datetime[0] == "2015"
                && datetime[1] == "07"
                && datetime[2] == "30"
                && datetime[3] == "03"
                && datetime[4] == "26"
                && datetime[5] == "13"
        }
        _ => false,
    }
}

fn parse_time_with_format(time: &str, format: &str) -> Result<NaiveDateTime> {
    let dt = NaiveDateTime::parse_from_str(time, format);
    match dt {
        Ok(dt) => Ok(dt),
        Err(NotEnough) => Err(
        Err(_) => Err(anyhow!("Failed to parse time: {} with format: {}", time, format)),
    }
}

fn time_to_unix(config: &Config, time: &str) -> Result<u64> {
    let tz: Tz = config.time_zone().parse().expect("Invalid time zone.");
    let naive_datetime =
        NaiveDateTime::parse_from_str(time, config.format()).with_context(|| {
            format!(
                "Failed to parse time: {} with format: {}",
                time,
                config.format()
            )
        })?;
    println!("Naive: {:?}", naive_datetime);

    // let aware_datetime = tz.with_ymd_and_hms(
    //     naive_datetime.year(),
    //     naive_datetime.month(),
    //     naive_datetime.day(),
    //     naive_datetime.hour(),
    //     naive_datetime.minute(),
    //     naive_datetime.second(),
    // );

    let time_components = split_time(time);

    let complete_time_components = if can_be_genesis(&time_components) {
        vec!["2015", "07", "30", "03", "26", "13"]
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
    let datetime = utc_datetime.with_timezone(&tz);

    Ok(datetime.timestamp() as u64)
}k

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

    fn get_test_config() -> Config {
        dotenv().ok();
        let rpc_url = dotenv!("RPC_URL");
        Config::new(
            rpc_url.to_string(),
            "UTC".to_string(),
            "%Y-%m-%d %H:%M:%S".to_string(),
        )
    }
    // stub of config
    #[tokio::test]
    async fn historical_block_to_time() {
        dotenv().ok();
        let rpc_url = dotenv!("RPC_URL");
        let config = Config::new(
            rpc_url.to_string(),
            "UTC".to_string(),
            "%Y-%m-%d %H:%M:%S".to_string(),
        );

        let known_time = 1438269988;
        let calculated_time = get_block_unix_time(&config, 1).await.unwrap();
        assert_eq!(known_time, calculated_time);
    }

    #[tokio::test]
    async fn future_block_to_time() {
        dotenv().ok();
        let rpc_url = dotenv!("RPC_URL");
        let config = Config::new(
            rpc_url.to_string(),
            "UTC".to_string(),
            "%Y-%m-%d %H:%M:%S".to_string(),
        );
        let provider = Provider::<Http>::try_from(config.rpc_url()).unwrap();

        let current_block = get_current_block_number(&provider).await.unwrap();
        let current_time = get_block_timestamp(&provider, current_block).await.unwrap();
        let block_num = current_block + 2;
        let estimated_time = get_block_unix_time(&config, block_num).await.unwrap();
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
        let time = vec!["2015", "07", "30", "04"];
        assert!(!can_be_genesis(&time));

        let time = vec!["2015", "07", "30", "03"];
        assert!(can_be_genesis(&time));

        // year, month, day, hour, minute
        let time = vec!["2015", "07", "30", "03", "27"];
        assert!(!can_be_genesis(&time));

        let time = vec!["2015", "07", "30", "03", "26"];
        assert!(can_be_genesis(&time));

        // year, month, day, hour, minute, second
        let time = vec!["2015", "07", "30", "03", "26", "14"];
        assert!(!can_be_genesis(&time));

        let time = vec!["2015", "07", "30", "03", "26", "13"];
        assert!(can_be_genesis(&time));
    }

    #[test]
    fn year_to_unix_genesis() {
        let time = "2015";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1438226773);
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
        assert_eq!(unix, 1438226773);
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
        assert_eq!(unix, 1438226773);
    }

    #[test]
    fn hour_to_unix_first_timestamp() {
        let time = "2016-02-01 01";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1454288400);
    }

    #[test]
    fn minute_to_unix_genesis() {
        let time = "2015-07-30 03:26";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1438226773);
    }

    #[test]
    fn minute_to_unix_first_timestamp() {
        let time = "2016-02-01 01:07";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1454288820);
    }

    #[test]
    fn second_to_unix_genesis() {
        let time = "2015-07-30 03:26:13";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1438226773);
    }

    #[test]
    fn second_to_unix_first_timestamp() {
        let time = "2016-02-01 01:07:05";
        let unix = time_to_unix(&get_test_config(), &time).unwrap();
        assert_eq!(unix, 1454288825);
    }
}
