use anyhow::{anyhow, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use rust_decimal::Decimal;
use std::str::FromStr;

use nyxforge_core::bond::{
    Bond, BondState, ComparisonOp, GoalMetric, GoalSpec, OracleSpec, VerificationCriteria,
};
use nyxforge_core::types::{Amount, PublicKey};

use crate::rpc_client::RpcClient;

// ---------------------------------------------------------------------------
// Clap types
// ---------------------------------------------------------------------------

#[derive(Parser)]
pub struct BondArgs {
    #[command(subcommand)]
    pub command: BondCommand,
}

#[derive(Subcommand)]
pub enum BondCommand {
    /// Create a new bond series via an interactive wizard
    Create,
    /// List all bond series known to the node
    List,
    /// Show full details for a single bond
    Get {
        /// Bond ID (32-byte hex)
        id: String,
    },
}

pub async fn run(args: BondArgs, rpc: &RpcClient) -> Result<()> {
    match args.command {
        BondCommand::Create  => cmd_create(rpc).await,
        BondCommand::List    => cmd_list(rpc).await,
        BondCommand::Get { id } => cmd_get(rpc, &id).await,
    }
}

// ---------------------------------------------------------------------------
// bond create — interactive wizard
// ---------------------------------------------------------------------------

async fn cmd_create(rpc: &RpcClient) -> Result<()> {
    let theme = ColorfulTheme::default();

    println!("\n{}", "  NyxForge — Create Bond".bold().purple());
    println!("  {}\n", "Answer each prompt. Press Enter to accept a default.".dimmed());

    // -- Goal spec -----------------------------------------------------------

    let title: String = Input::with_theme(&theme)
        .with_prompt("Goal title")
        .interact_text()?;

    let description: String = Input::with_theme(&theme)
        .with_prompt("Goal description")
        .interact_text()?;

    let data_id: String = Input::with_theme(&theme)
        .with_prompt("Data ID")
        .default("us.hud.pit_count".into())
        .interact_text()?;

    let op_labels = &["lt  (less than)", "lte (less than or equal)", "gt  (greater than)",
                      "gte (greater than or equal)", "eq  (equal)"];
    let op_idx = Select::with_theme(&theme)
        .with_prompt("Operator")
        .items(op_labels)
        .default(0)
        .interact()?;
    let operator = [
        ComparisonOp::LessThan, ComparisonOp::LessThanOrEqual,
        ComparisonOp::GreaterThan, ComparisonOp::GreaterThanOrEqual,
        ComparisonOp::Equal,
    ][op_idx].clone();

    let threshold: Decimal = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Threshold value")
            .interact_text()?;
        match Decimal::from_str(&s) {
            Ok(d) => break d,
            Err(_) => eprintln!("  {} Enter a valid decimal number.", "!".red()),
        }
    };

    let aggregation: String = Input::with_theme(&theme)
        .with_prompt("Aggregation (optional, Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let evidence_format: String = Input::with_theme(&theme)
        .with_prompt("Evidence format (optional, Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    let deadline = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Deadline (YYYY-MM-DD)")
            .interact_text()?;
        match NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            Ok(d) => break Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()),
            Err(_) => eprintln!("  {} Use YYYY-MM-DD format (e.g. 2030-01-01).", "!".red()),
        }
    };

    // -- Bond economics ------------------------------------------------------

    println!("\n  {}", "Economics".bold());

    let total_supply: u64 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Total supply (number of bonds to issue)")
            .interact_text()?;
        match s.parse::<u64>() {
            Ok(n) if n > 0 => break n,
            _ => eprintln!("  {} Must be a positive integer.", "!".red()),
        }
    };

    let redemption_drk: u64 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Redemption value per bond (whole DRK)")
            .interact_text()?;
        match s.parse::<u64>() {
            Ok(n) => break n,
            _ => eprintln!("  {} Must be a non-negative integer.", "!".red()),
        }
    };

    let floor_drk: u64 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Floor price per bond (whole DRK)")
            .interact_text()?;
        match s.parse::<u64>() {
            Ok(n) => break n,
            _ => eprintln!("  {} Must be a non-negative integer.", "!".red()),
        }
    };

    // -- Oracle spec ---------------------------------------------------------

    println!("\n  {}", "Oracle network".bold());

    let quorum: u32 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Oracle quorum (min independent attestations required)")
            .default("3".into())
            .interact_text()?;
        match s.parse::<u32>() {
            Ok(n) if n > 0 => break n,
            _ => eprintln!("  {} Must be >= 1.", "!".red()),
        }
    };

    // Fetch wallet DRK pubkey as the default oracle key.
    let default_oracle_key = fetch_drk_pubkey(rpc).await;
    let oracle_keys_prompt = match &default_oracle_key {
        Some(k) => format!("Oracle public keys (hex, comma-separated) [default: your DRK key {}]", &k[..8]),
        None    => "Oracle public keys (hex, comma-separated)".into(),
    };
    let oracle_keys_raw: String = Input::with_theme(&theme)
        .with_prompt(oracle_keys_prompt)
        .allow_empty(default_oracle_key.is_some())
        .interact_text()?;

    let oracle_keys: Vec<PublicKey> = if oracle_keys_raw.trim().is_empty() {
        match default_oracle_key {
            Some(k) => vec![parse_pubkey(&k)?],
            None    => return Err(anyhow!("No oracle key provided and no wallet found. Create a wallet first.")),
        }
    } else {
        oracle_keys_raw.split(',')
            .map(|s| parse_pubkey(s.trim()))
            .collect::<Result<_>>()?
    };

    let required_stake_drk: u64 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Required oracle stake (whole DRK)")
            .default("100".into())
            .interact_text()?;
        match s.parse::<u64>() {
            Ok(n) => break n,
            _ => eprintln!("  {} Must be a non-negative integer.", "!".red()),
        }
    };

    let slash_fraction: Decimal = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Slash fraction for fraud (0.0 – 1.0)")
            .default("0.5".into())
            .interact_text()?;
        match Decimal::from_str(&s) {
            Ok(d) if d >= Decimal::ZERO && d <= Decimal::ONE => break d,
            _ => eprintln!("  {} Must be between 0.0 and 1.0.", "!".red()),
        }
    };

    // -- Verification criteria -----------------------------------------------

    println!("\n  {}", "Verification".bold());

    let attestation_threshold: u32 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Attestation threshold (must be <= quorum)")
            .default(quorum.to_string())
            .interact_text()?;
        match s.parse::<u32>() {
            Ok(n) if n > 0 && n <= quorum => break n,
            _ => eprintln!("  {} Must be between 1 and {}.", "!".red(), quorum),
        }
    };

    let challenge_period: u64 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Challenge period (seconds)")
            .default("86400".into())
            .interact_text()?;
        match s.parse::<u64>() {
            Ok(n) => break n,
            _ => eprintln!("  {} Must be a non-negative integer.", "!".red()),
        }
    };

    let dao_override = Confirm::with_theme(&theme)
        .with_prompt("Allow DAO governance to override oracle consensus?")
        .default(false)
        .interact()?;

    // -- Build bond ----------------------------------------------------------

    let issuer = fetch_issuer_pubkey(rpc).await?;

    let goal = GoalSpec {
        title,
        description,
        metric: GoalMetric {
            data_id,
            operator,
            threshold,
            aggregation: if aggregation.is_empty() { None } else { Some(aggregation) },
        },
        evidence_format: if evidence_format.is_empty() { None } else { Some(evidence_format) },
        deadline,
    };

    let id = Bond::compute_id(&goal, &issuer, 0);

    let bond = Bond {
        id,
        issuer,
        total_supply,
        redemption_value: Amount::from_whole(redemption_drk),
        floor_price: Amount::from_whole(floor_drk),
        state: BondState::Draft,
        goal,
        oracle: OracleSpec {
            quorum,
            oracle_keys,
            required_stake: Amount::from_whole(required_stake_drk),
            slash_fraction,
        },
        verification: VerificationCriteria {
            attestation_threshold,
            challenge_period_secs: challenge_period,
            dao_override_allowed: dao_override,
        },
        created_at_block: 0,
    };

    // -- Preview + confirm ---------------------------------------------------

    println!("\n{}", "  --- Bond preview ---".bold());
    println!("{}", serde_json::to_string_pretty(&bond)?);

    println!();
    if !Confirm::with_theme(&theme)
        .with_prompt("Submit this bond?")
        .default(false)
        .interact()?
    {
        println!("{}", "  Cancelled.".dimmed());
        return Ok(());
    }

    // -- Submit --------------------------------------------------------------

    let result = rpc.call("bonds.issue", serde_json::json!({ "bond": bond })).await?;
    let bond_id = result["bond_id"].as_str().unwrap_or("(unknown)");
    println!("\n  {} Bond issued: {}", "✔".green().bold(), bond_id.cyan().bold());
    Ok(())
}

// ---------------------------------------------------------------------------
// bond list
// ---------------------------------------------------------------------------

async fn cmd_list(rpc: &RpcClient) -> Result<()> {
    let result = rpc.call("bonds.list", serde_json::json!({})).await?;
    let bonds = result["bonds"]
        .as_array()
        .ok_or_else(|| anyhow!("unexpected response from bonds.list"))?;

    if bonds.is_empty() {
        println!("{}", "  No bonds found.".dimmed());
        return Ok(());
    }

    println!("\n  {:<66}  {:<12}  {}", "Bond ID".bold(), "State".bold(), "Title".bold());
    println!("  {}", "-".repeat(100).dimmed());

    for b in bonds {
        let id    = b["id"].as_object()
            .and_then(|o| o.values().next())
            .and_then(|v| v.as_array())
            .map(|arr| hex::encode(arr.iter().filter_map(|x| x.as_u64()).map(|x| x as u8).collect::<Vec<_>>()))
            .unwrap_or_else(|| serde_json::to_string(&b["id"]).unwrap_or_default());
        let state = b["state"].as_str().unwrap_or("?");
        let title = b["goal"]["title"].as_str().unwrap_or("?");
        println!("  {:<66}  {:<12}  {}", id.cyan(), state, title);
    }
    println!();
    Ok(())
}

// ---------------------------------------------------------------------------
// bond get
// ---------------------------------------------------------------------------

async fn cmd_get(rpc: &RpcClient, id: &str) -> Result<()> {
    let result = rpc.call("bonds.get", serde_json::json!({ "id": id })).await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn fetch_drk_pubkey(rpc: &RpcClient) -> Option<String> {
    rpc.call("wallet.addresses", serde_json::json!({}))
        .await
        .ok()
        .and_then(|v| v["drk"].as_str().map(str::to_owned))
}

async fn fetch_issuer_pubkey(rpc: &RpcClient) -> Result<PublicKey> {
    let drk_hex = fetch_drk_pubkey(rpc)
        .await
        .ok_or_else(|| anyhow!("No wallet found. Create one first with: nyxforge-cli wallet create"))?;
    parse_pubkey(&drk_hex)
}

fn parse_pubkey(hex_str: &str) -> Result<PublicKey> {
    let bytes = hex::decode(hex_str.trim())
        .map_err(|e| anyhow!("Invalid hex key '{hex_str}': {e}"))?;
    if bytes.len() != 32 {
        return Err(anyhow!("Public key must be 32 bytes, got {}", bytes.len()));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(PublicKey(arr))
}
