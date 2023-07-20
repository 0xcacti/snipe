use std::process;

use clap::{crate_version, Parser, Subcommand};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use snipe::Config;

/// bible-rs is a command line tool for getting a random verse from the Bible.
#[derive(Debug, Parser)]
#[command(name="snipe", version=crate_version!(), about="blocktime estimator for ethereum mainnet", long_about = "Convert blocknumber to approximate time, and time to approximate blocknumber", arg_required_else_help(true))]
struct CLIParser {
    // The subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,
    /// The rpc url to use
    #[arg(short, long, required = false, global = true)]
    rpc_url: Option<String>,

    /// The timezone to use
    #[arg(short, long, required = false, global = true)]
    timezone: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// time to blocknumber
    #[command(
        name = "block-to-time",
        about = "time to blocknumber",
        visible_alias = "btt"
    )]
    BlockToTime {
        /// The blocknumber to convert
        #[arg(required = true)]
        block_num: u64,
    },
    /// blocknumber to time
    #[command(
        name = "time-to-block",
        about = "time to blocknumber",
        visible_alias = "ttb"
    )]
    TimeToBlock {
        /// The time to Convert
        #[arg(required = true)]
        time: String,
    },
    /// get all available timezones
    #[command(
        name = "list-timezones",
        about = "get all available timezones",
        visible_alias = "tz"
    )]
    ListTimezones,
}

#[tokio::main]
async fn main() {
    let mut config: Config = Figment::new()
        .merge(Toml::file("snipe.toml"))
        .merge(Toml::file("foundry.toml").nested())
        .merge(Env::prefixed("SNIPE_"))
        .extract()
        .unwrap();

    let args = CLIParser::parse();
    // Check for Bible version
    match args.rpc_url {
        Some(rpc_url) => config.rpc_url = Some(rpc_url),
        None => match config.rpc_url {
            Some(rpc_url) => config.rpc_url = Some(rpc_url),
            None => {
                eprintln!("No RPC URL provided. Please provide a RPC URL using the --rpc-url flag, setting rpc_url in the snipe.toml file, or by setting the SNIPE_RPC_URL environment variable.");
                process::exit(1);
            }
        },
    }

    match args.timezone {
        Some(timezone) => config.time_zone = Some(timezone),
        None => match config.time_zone {
            Some(timezone) => config.time_zone = Some(timezone),
            None => config.time_zone = Some("UTC".to_string()),
        },
    }

    // handle commands
    match &args.command {
        Some(Commands::BlockToTime { block_num }) => {
            let time = snipe::block_to_time(config, *block_num).await;
            match time {
                Ok(time) => {
                    println!("{}", time);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }

        Some(Commands::TimeToBlock { time }) => {
            let block = snipe::time_to_block(config, time).unwrap();
        }
        Some(Commands::ListTimezones) => {
            let timezones = snipe::list_timezones();
        }
        None => {}
    }
}
