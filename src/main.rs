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

    /// The format to use
    #[arg(short, long, required = false, global = true)]
    format: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// blocknumber to time
    #[command(
        name = "block-to-time",
        about = "blocknumber to time",
        visible_alias = "btt"
    )]
    BlockToTime {
        /// The blocknumber to convert
        #[arg(required = true)]
        block_num: u64,
    },

    /// time to time
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
        .merge(Toml::file("foundry.toml"))
        .merge(Env::prefixed("SNIPE_"))
        .extract()
        .unwrap();

    let args = CLIParser::parse();

    match args.rpc_url {
        Some(rpc_url) => config.snipe_rpc_url = Some(rpc_url),
        None => match config.snipe_rpc_url {
            Some(rpc_url) => config.snipe_rpc_url = Some(rpc_url),
            None => {
                eprintln!("No RPC URL provided. Please provide a RPC URL using the --rpc-url flag, setting snipe_rpc_url in a foundry.toml file, or by setting the SNIPE_RPC_URL environment variable.");
                process::exit(1);
            }
        },
    }

    config.time_zone = args
        .timezone
        .or_else(|| config.time_zone)
        .or_else(|| Some("UTC".to_string()));

    // handle commandsk
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
            let block = snipe::time_to_block(&config, time).await;
            match block {
                Ok(block) => {
                    println!("{}", block);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        Some(Commands::ListTimezones) => {
            snipe::list_timezones();
        }
        None => {}
    }
}
