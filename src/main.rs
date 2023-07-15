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
    /// The subcommand to run
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
    #[command(name = "time", alias = "t", about = "time to blocknumber [aliases: t]")]
    GetBlockNumber,
    /// blocknumber to time
    GetTime,
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

    // handle commands
    match &args.command {
        Some(GetBlockNumber) => {
            println!("No command specified");
        }
        None => {
            println!("No command specified");
        }
    }
}
