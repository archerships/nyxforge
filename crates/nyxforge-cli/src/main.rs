use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod mcp_client;
mod rpc_client;

use mcp_client::McpClient;
use rpc_client::RpcClient;

/// NyxForge bearer bounty toolkit
#[derive(Parser)]
#[command(name = "nyxforge-cli", version, about = "Create, inspect, and settle .bounty files")]
struct Cli {
    /// Node RPC URL (used only by dev commands; not required for file operations)
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
    /// Bounty file operations (create, inspect, transfer, issue, redeem, reclaim, verify)
    Bounty(commands::bounty::BountyArgs),
    /// Judge operations (accept, reject, attest, status, list)
    Judge(commands::judge::JudgeArgs),
    /// AI provider management (via MCP server)
    Mcp(commands::mcp::McpArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let rpc = RpcClient::new(&cli.rpc);
    let mcp = McpClient::new(&cli.mcp);

    match cli.command {
        Commands::Bounty(args)   => commands::bounty::run(args, &rpc, &mcp).await,
        Commands::Judge(args) => commands::judge::run(args, &rpc).await,
        Commands::Mcp(args)    => commands::mcp::run(args, &mcp).await,
    }
}
