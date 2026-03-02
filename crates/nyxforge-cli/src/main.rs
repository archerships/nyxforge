use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod rpc_client;

use rpc_client::RpcClient;

/// NyxForge command-line interface
#[derive(Parser)]
#[command(name = "nyxforge-cli", version, about = "Interact with a running NyxForge node")]
struct Cli {
    /// Node JSON-RPC URL
    #[arg(long, default_value = "http://127.0.0.1:8888/rpc", env = "NYXFORGE_RPC")]
    rpc: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bond management
    Bond(commands::bond::BondArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let rpc = RpcClient::new(&cli.rpc);

    match cli.command {
        Commands::Bond(args) => commands::bond::run(args, &rpc).await,
    }
}
