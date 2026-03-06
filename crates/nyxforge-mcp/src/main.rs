//! NyxForge MCP server — AI-agnostic bridge between the CLI and any AI provider.
//!
//! Usage:
//!   nyxforge-mcp [--bind 127.0.0.1:8890]
//!
//! The server reads provider configuration from
//! `~/.config/nyxforge/mcp.json` and exposes:
//!
//!   POST /               JSON-RPC (MCP protocol)
//!   GET  /providers      list configured providers
//!   POST /providers      add a provider
//!   DELETE /providers/:name  remove a provider
//!   PUT  /providers/default  set the default provider
//!   GET  /health         health check

mod config;
mod provider;
mod server;

use std::sync::Arc;
use tokio::sync::RwLock;

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

use config::McpConfig;

#[derive(Parser)]
#[command(name = "nyxforge-mcp", version, about = "NyxForge AI-provider bridge (MCP server)")]
struct Cli {
    /// Address to listen on.
    #[arg(long, default_value = "127.0.0.1:8890", env = "NYXFORGE_MCP_BIND")]
    bind: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("nyxforge_mcp=info")),
        )
        .init();

    let cli = Cli::parse();
    let cfg = McpConfig::load()?;

    if cfg.providers.is_empty() {
        info!(
            "No providers configured yet. \
             Add one with: nyxforge-cli mcp add <name> --kind <anthropic|openai|ollama|custom>"
        );
    } else {
        let default = cfg.default_provider.as_deref().unwrap_or("(none)");
        info!(
            "Loaded {} provider(s). Default: {}",
            cfg.providers.len(),
            default
        );
    }

    let state = Arc::new(RwLock::new(cfg));
    server::serve(&cli.bind, state).await
}
