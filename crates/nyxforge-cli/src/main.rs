use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod mcp_client;
mod rpc_client;

use mcp_client::McpClient;
use rpc_client::RpcClient;

/// NyxForge command-line interface
#[derive(Parser)]
#[command(name = "nyxforge-cli", version, about = "Interact with a running NyxForge node")]
struct Cli {
    /// Node JSON-RPC URL
    #[arg(long, default_value = "http://127.0.0.1:8888/rpc", env = "NYXFORGE_RPC")]
    rpc: String,

    /// MCP server URL (AI provider bridge)
    #[arg(long, default_value = "http://127.0.0.1:8890", env = "NYXFORGE_MCP")]
    mcp: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bond management
    Bond(commands::bond::BondArgs),
    /// AI provider management (via MCP server)
    Mcp(commands::mcp::McpArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let rpc = RpcClient::new(&cli.rpc);
    let mcp = McpClient::new(&cli.mcp);

    match cli.command {
        Commands::Bond(args) => commands::bond::run(args, &rpc, &mcp).await,
        Commands::Mcp(args)  => commands::mcp::run(args, &mcp).await,
    }
}
