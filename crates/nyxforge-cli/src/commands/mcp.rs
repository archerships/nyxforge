//! `nyxforge-cli mcp` — manage AI providers via the MCP server.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};

use crate::mcp_client::McpClient;

// ---------------------------------------------------------------------------
// Clap types
// ---------------------------------------------------------------------------

#[derive(Parser)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(Subcommand)]
pub enum McpCommand {
    /// Show MCP server status and the active provider
    Status,
    /// List all configured AI providers
    Providers,
    /// Add an AI provider interactively
    Add {
        /// Short name for this provider (e.g. "claude", "gpt4", "local")
        name: String,
        /// Provider kind: anthropic | openai | ollama | custom
        #[arg(long)]
        kind: Option<String>,
    },
    /// Remove a configured provider
    Remove {
        /// Provider name to remove
        name: String,
    },
    /// Set the default (active) provider
    Default {
        /// Provider name to use as default
        name: String,
    },
}

pub async fn run(args: McpArgs, mcp: &McpClient) -> Result<()> {
    match args.command {
        McpCommand::Status          => cmd_status(mcp).await,
        McpCommand::Providers       => cmd_providers(mcp).await,
        McpCommand::Add { name, kind } => cmd_add(mcp, &name, kind.as_deref()).await,
        McpCommand::Remove { name } => cmd_remove(mcp, &name).await,
        McpCommand::Default { name} => cmd_set_default(mcp, &name).await,
    }
}

// ---------------------------------------------------------------------------
// mcp status
// ---------------------------------------------------------------------------

async fn cmd_status(mcp: &McpClient) -> Result<()> {
    match mcp.health().await {
        Ok(_)  => println!("  {} MCP server is running at {}", "✔".green().bold(), "127.0.0.1:8890".cyan()),
        Err(e) => {
            println!("  {} MCP server not reachable.", "✘".red().bold());
            println!("  Start it with: {}", "nyxforge-mcp".yellow());
            println!("  Error: {e}");
            return Ok(());
        }
    }

    let info = mcp.providers().await?;
    let providers = info["providers"].as_array().cloned().unwrap_or_default();
    if providers.is_empty() {
        println!("\n  {}", "No providers configured. Use: nyxforge-cli mcp add".dimmed());
    } else {
        println!();
        for p in &providers {
            let name      = p["name"].as_str().unwrap_or("?");
            let kind      = p["kind"].as_str().unwrap_or("?");
            let model     = p["model"].as_str().unwrap_or("?");
            let is_def    = p["is_default"].as_bool().unwrap_or(false);
            let has_key   = p["has_api_key"].as_bool().unwrap_or(false);
            let def_label = if is_def { " (default)".green().bold().to_string() } else { String::new() };
            let key_label = if has_key { "key set".dimmed().to_string() } else { "no key".yellow().to_string() };
            println!("  {}{}  —  {} / {}  [{}]", name.bold(), def_label, kind, model, key_label);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// mcp providers
// ---------------------------------------------------------------------------

async fn cmd_providers(mcp: &McpClient) -> Result<()> {
    let info = mcp.providers().await?;
    let providers = info["providers"].as_array().cloned().unwrap_or_default();

    if providers.is_empty() {
        println!("{}", "  No providers configured.".dimmed());
        println!("  Add one with: {}", "nyxforge-cli mcp add <name>".yellow());
        return Ok(());
    }

    println!("\n  {:<20}  {:<12}  {:<30}  {}", "Name".bold(), "Kind".bold(), "Model".bold(), "Default".bold());
    println!("  {}", "-".repeat(80).dimmed());
    for p in &providers {
        let name  = p["name"].as_str().unwrap_or("?");
        let kind  = p["kind"].as_str().unwrap_or("?");
        let model = p["model"].as_str().unwrap_or("?");
        let def   = if p["is_default"].as_bool().unwrap_or(false) { "✔" } else { "" };
        println!("  {:<20}  {:<12}  {:<30}  {}", name, kind, model, def.green());
    }
    println!();
    Ok(())
}

// ---------------------------------------------------------------------------
// mcp add
// ---------------------------------------------------------------------------

async fn cmd_add(mcp: &McpClient, name: &str, kind_arg: Option<&str>) -> Result<()> {
    let theme = ColorfulTheme::default();

    println!("\n{}", "  NyxForge — Add AI Provider".bold().purple());

    // -- Kind ----------------------------------------------------------------

    let kind_options = &["anthropic", "openai", "ollama", "custom"];
    let kind_default = kind_arg
        .and_then(|k| kind_options.iter().position(|&o| o == k))
        .unwrap_or(0);
    let kind_idx = Select::with_theme(&theme)
        .with_prompt("Provider kind")
        .items(kind_options)
        .default(kind_default)
        .interact()?;
    let kind = kind_options[kind_idx];

    // -- API key (Anthropic / OpenAI / Custom) --------------------------------

    let api_key: Option<String> = match kind {
        "ollama" => None,
        _ => {
            let key: String = Password::with_theme(&theme)
                .with_prompt(format!("{kind} API key (leave blank to skip)"))
                .allow_empty_password(true)
                .interact()?;
            if key.is_empty() { None } else { Some(key) }
        }
    };

    // -- Base URL (Ollama / Custom) -------------------------------------------

    let base_url: Option<String> = match kind {
        "ollama" => {
            let u: String = Input::with_theme(&theme)
                .with_prompt("Ollama base URL")
                .default("http://localhost:11434".into())
                .interact_text()?;
            Some(u)
        }
        "custom" => {
            let u: String = Input::with_theme(&theme)
                .with_prompt("Base URL (OpenAI-compatible endpoint)")
                .interact_text()?;
            Some(u)
        }
        _ => None,
    };

    // -- Model ---------------------------------------------------------------

    let model_default = match kind {
        "anthropic" => "claude-sonnet-4-6",
        "openai"    => "gpt-4o",
        "ollama"    => "llama3",
        _           => "",
    };
    let model: String = Input::with_theme(&theme)
        .with_prompt("Model name")
        .default(model_default.into())
        .interact_text()?;
    let model = if model.is_empty() || model == model_default { None } else { Some(model.as_str()) };

    // -- Submit --------------------------------------------------------------

    let result = mcp.add_provider(
        name,
        kind,
        api_key.as_deref(),
        base_url.as_deref(),
        model,
    ).await?;

    if result["ok"].as_bool().unwrap_or(false) {
        println!("\n  {} Provider '{}' added.", "✔".green().bold(), name.cyan().bold());
        println!("  {}", "It has been set as the default if it was the first provider.".dimmed());
    } else {
        println!("\n  {} {}", "✘".red().bold(), result);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// mcp remove
// ---------------------------------------------------------------------------

async fn cmd_remove(mcp: &McpClient, name: &str) -> Result<()> {
    let result = mcp.remove_provider(name).await?;
    if result["ok"].as_bool().unwrap_or(false) {
        println!("  {} Provider '{}' removed.", "✔".green().bold(), name.cyan());
    } else {
        let err = result["error"].as_str().unwrap_or("unknown error");
        println!("  {} {}", "✘".red().bold(), err);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// mcp default
// ---------------------------------------------------------------------------

async fn cmd_set_default(mcp: &McpClient, name: &str) -> Result<()> {
    let result = mcp.set_default(name).await?;
    if result["ok"].as_bool().unwrap_or(false) {
        println!("  {} Default provider set to '{}'.", "✔".green().bold(), name.cyan().bold());
    } else {
        let err = result["error"].as_str().unwrap_or("unknown error");
        println!("  {} {}", "✘".red().bold(), err);
    }
    Ok(())
}
