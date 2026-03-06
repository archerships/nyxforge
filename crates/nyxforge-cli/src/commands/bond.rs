use anyhow::{anyhow, Result};
use chrono::{NaiveDate, TimeZone, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use rust_decimal::Decimal;
use std::str::FromStr;

use nyxforge_core::bond::{
    AuctionParams, Bond, BondState, ComparisonOp, GoalMetric, GoalSpec, OracleSpec,
    VerificationCriteria,
};
use nyxforge_core::types::{Amount, PublicKey};

use crate::mcp_client::McpClient;
use crate::rpc_client::RpcClient;

// ---------------------------------------------------------------------------
// AI wizard defaults
// ---------------------------------------------------------------------------

/// Pre-fill values supplied by the AI assistant.  All fields shown to the
/// user with `with_initial_text` so they can see and edit the suggestions.
pub struct BondDefaults {
    pub title:           String,
    pub description:     String,
    pub data_id:         String,
    pub operator_idx:    usize,   // 0=lt 1=lte 2=gt 3=gte 4=eq
    pub threshold:       String,
    pub aggregation:     String,
    pub evidence_format: String,
    pub deadline:        String,  // YYYY-MM-DD
}

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
    /// Publish a bond proposal for community review (no collateral locked)
    Propose,
    /// Submit a bond for oracle acceptance (from Proposed state or fresh wizard)
    Submit {
        /// Bond ID of an existing Proposed bond (omit to run wizard instead)
        id: Option<String>,
    },
    /// Submit a bond wizard result directly for oracle approval, then lock
    /// collateral once approved (skips community proposal step)
    Create,
    /// Lock collateral and activate a Draft bond that has oracle approval
    Issue {
        /// Bond ID (32-byte hex)
        id: String,
    },
    /// Accept responsibility for judging a bond (run as an oracle operator)
    OracleAccept {
        /// Bond ID (32-byte hex)
        id: String,
    },
    /// Decline to judge a bond with a reason (run as an oracle operator)
    OracleReject {
        /// Bond ID (32-byte hex)
        id: String,
    },
    /// Show each oracle's accept/reject status for a bond
    OracleStatus {
        /// Bond ID (32-byte hex)
        id: String,
    },
    /// Replace the oracle list on a pending bond after rejections
    ReviseOracles {
        /// Bond ID (32-byte hex)
        id: String,
    },
    /// List all bond series known to the node
    List,
    /// Show full details for a single bond
    Get {
        /// Bond ID (32-byte hex)
        id: String,
    },
    /// Post a question or suggestion on a proposed bond
    Comment {
        /// Bond ID (32-byte hex)
        id: String,
    },
    /// List all comments on a bond
    Comments {
        /// Bond ID (32-byte hex)
        id: String,
    },
    /// Describe your goal in plain language; AI finds similar bonds and drafts a new one
    Explore,
    /// Show the current Dutch auction ask price for an active bond
    AuctionPrice {
        /// Bond ID (32-byte hex)
        id: String,
    },
    /// Buy bonds at the current auction price
    Buy {
        /// Bond ID (32-byte hex)
        id: String,
        /// Number of bonds to purchase
        quantity: u64,
    },
}

pub async fn run(args: BondArgs, rpc: &RpcClient, mcp: &McpClient) -> Result<()> {
    match args.command {
        BondCommand::Explore                    => cmd_explore(rpc, mcp).await,
        BondCommand::Propose                    => cmd_propose(rpc).await,
        BondCommand::Submit { id }              => cmd_submit(rpc, id.as_deref()).await,
        BondCommand::Create                     => cmd_create(rpc).await,
        BondCommand::Issue { id }               => cmd_issue(rpc, &id).await,
        BondCommand::OracleAccept { id }        => cmd_oracle_accept(rpc, &id).await,
        BondCommand::OracleReject { id }        => cmd_oracle_reject(rpc, &id).await,
        BondCommand::OracleStatus { id }        => cmd_oracle_status(rpc, &id).await,
        BondCommand::ReviseOracles { id }       => cmd_revise_oracles(rpc, &id).await,
        BondCommand::List                       => cmd_list(rpc).await,
        BondCommand::Get { id }                 => cmd_get(rpc, &id).await,
        BondCommand::Comment { id }             => cmd_comment(rpc, &id).await,
        BondCommand::Comments { id }            => cmd_comments(rpc, &id).await,
        BondCommand::AuctionPrice { id }        => cmd_auction_price(rpc, &id).await,
        BondCommand::Buy { id, quantity }       => cmd_buy(rpc, &id, quantity).await,
    }
}

// ---------------------------------------------------------------------------
// Shared bond wizard — used by both `propose` and `create`
// ---------------------------------------------------------------------------

/// Run the interactive bond wizard and return the completed Bond.
/// Pass `defaults` (from the AI assistant) to pre-fill the prompts.
async fn run_wizard(rpc: &RpcClient, defaults: Option<&BondDefaults>) -> Result<Bond> {
    let theme = ColorfulTheme::default();

    if defaults.is_some() {
        println!("\n  {}\n", "AI suggestions are pre-filled — edit freely, then press Enter.".dimmed());
    } else {
        println!("\n  {}\n", "Answer each prompt. Press Enter to accept a default.".dimmed());
    }

    // -- Goal criteria (one or more; AND semantics) --------------------------

    let mut goal_specs: Vec<GoalSpec> = Vec::new();
    let mut first = true;
    loop {
        let criterion_num = goal_specs.len() + 1;
        if criterion_num > 1 {
            println!("\n  {}", format!("Criterion {criterion_num}").bold());
        }

        // For the first criterion, seed defaults from the AI assistant if given.
        let use_defaults = first;

        let mut title_input = Input::<String>::with_theme(&theme).with_prompt("Criterion title");
        if use_defaults { if let Some(d) = defaults { title_input = title_input.with_initial_text(&d.title); } }
        let title: String = title_input.interact_text()?;

        let mut desc_input = Input::<String>::with_theme(&theme).with_prompt("Description");
        if use_defaults { if let Some(d) = defaults { desc_input = desc_input.with_initial_text(&d.description); } }
        let description: String = desc_input.interact_text()?;

        let mut data_id_input = Input::<String>::with_theme(&theme)
            .with_prompt("Data ID")
            .default("us.hud.pit_count".into());
        if use_defaults { if let Some(d) = defaults { data_id_input = data_id_input.with_initial_text(&d.data_id); } }
        let data_id: String = data_id_input.interact_text()?;

        let op_labels = &["lt  (less than)", "lte (less than or equal)", "gt  (greater than)",
                          "gte (greater than or equal)", "eq  (equal)"];
        let op_default = if use_defaults { defaults.map(|d| d.operator_idx).unwrap_or(0) } else { 0 };
        let op_idx = Select::with_theme(&theme)
            .with_prompt("Operator")
            .items(op_labels)
            .default(op_default)
            .interact()?;
        let operator = [
            ComparisonOp::LessThan, ComparisonOp::LessThanOrEqual,
            ComparisonOp::GreaterThan, ComparisonOp::GreaterThanOrEqual,
            ComparisonOp::Equal,
        ][op_idx].clone();

        let threshold: Decimal = loop {
            let mut t_input = Input::<String>::with_theme(&theme).with_prompt("Threshold value");
            if use_defaults { if let Some(d) = defaults { t_input = t_input.with_initial_text(&d.threshold); } }
            let s: String = t_input.interact_text()?;
            match Decimal::from_str(&s) {
                Ok(d) => break d,
                Err(_) => eprintln!("  {} Enter a valid decimal number.", "!".red()),
            }
        };

        let mut agg_input = Input::<String>::with_theme(&theme)
            .with_prompt("Aggregation (optional, Enter to skip)")
            .allow_empty(true);
        if use_defaults { if let Some(d) = defaults { agg_input = agg_input.with_initial_text(&d.aggregation); } }
        let aggregation: String = agg_input.interact_text()?;

        let mut ev_input = Input::<String>::with_theme(&theme)
            .with_prompt("Evidence format (optional, Enter to skip)")
            .allow_empty(true);
        if use_defaults { if let Some(d) = defaults { ev_input = ev_input.with_initial_text(&d.evidence_format); } }
        let evidence_format: String = ev_input.interact_text()?;

        let deadline = loop {
            let mut dl_input = Input::<String>::with_theme(&theme).with_prompt("Deadline (YYYY-MM-DD)");
            if use_defaults { if let Some(d) = defaults { dl_input = dl_input.with_initial_text(&d.deadline); } }
            let s: String = dl_input.interact_text()?;
            match NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                Ok(d) => break Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()),
                Err(_) => eprintln!("  {} Use YYYY-MM-DD format (e.g. 2030-01-01).", "!".red()),
            }
        };

        goal_specs.push(GoalSpec {
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
        });

        first = false;

        let add_more = Confirm::with_theme(&theme)
            .with_prompt("Add another criterion? [y/N]")
            .default(false)
            .interact()?;
        if !add_more { break; }
    }

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

    // 1. Starting price
    let start_drk: u64 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Auction starting price per bond (whole DRK)")
            .interact_text()?;
        match s.parse::<u64>() {
            Ok(n) if n > 0 => break n,
            _ => eprintln!("  {} Must be a positive integer.", "!".red()),
        }
    };

    // 2. Reserve price (≤ start)
    let reserve_drk: u64 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Reserve (floor) price per bond (whole DRK)")
            .interact_text()?;
        match s.parse::<u64>() {
            Ok(n) if n <= start_drk => break n,
            Ok(_) => eprintln!("  {} Reserve must be ≤ starting price ({start_drk} DRK).", "!".red()),
            _ => eprintln!("  {} Must be a non-negative integer.", "!".red()),
        }
    };

    // 3. Auction duration in days (default 7)
    let auction_days: u64 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Auction duration in days [7]")
            .default("7".to_string())
            .interact_text()?;
        match s.parse::<u64>() {
            Ok(n) if n > 0 => break n,
            _ => eprintln!("  {} Must be a positive integer.", "!".red()),
        }
    };
    let auction_secs = auction_days * 86_400;

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

    println!("\n  {}", "Addresses".bold());

    let return_address_raw: String = Input::with_theme(&theme)
        .with_prompt("Return address — collateral destination if goal not met (hex pubkey)")
        .interact_text()?;
    let return_address = parse_pubkey(return_address_raw.trim())?;

    let id = Bond::compute_id(&goal_specs, &issuer, 0, &return_address);

    let bond = Bond {
        id,
        issuer,
        total_supply,
        redemption_value:  Amount::from_whole(redemption_drk),
        auction:           AuctionParams {
            start_price:   Amount::from_whole(start_drk),
            reserve_price: Amount::from_whole(reserve_drk),
            duration_secs: auction_secs,
        },
        bonds_remaining:   total_supply,
        activated_at_secs: None,
        state:             BondState::Draft, // caller will override for proposals
        goals: goal_specs,
        oracle: OracleSpec {
            quorum,
            oracle_keys,
            required_stake: Amount::from_whole(required_stake_drk),
            slash_fraction,
        },
        verification: VerificationCriteria {
            attestation_threshold,
            challenge_period_secs: challenge_period,
            dao_override_allowed:  dao_override,
        },
        created_at_block: 0,
        return_address,
    };

    Ok(bond)
}

// ---------------------------------------------------------------------------
// bond explore — AI-assisted bond discovery and drafting
// ---------------------------------------------------------------------------

async fn cmd_explore(rpc: &RpcClient, mcp: &McpClient) -> Result<()> {
    let theme = ColorfulTheme::default();

    println!("\n{}", "  NyxForge — Bond Explorer".bold().purple());
    println!("  {}", "Describe your goal and AI will find similar bonds and draft a new one.".dimmed());
    println!("  {}\n", "Requires nyxforge-mcp to be running with a provider configured.".dimmed());

    // -- Get natural-language description ------------------------------------

    let description: String = Input::with_theme(&theme)
        .with_prompt("Describe the social or environmental goal you'd like to fund")
        .interact_text()?;
    if description.trim().is_empty() {
        println!("{}", "  Cancelled — empty description.".dimmed());
        return Ok(());
    }

    // -- Fetch existing bonds ------------------------------------------------

    print!("  {} Fetching existing bonds…", "→".cyan());
    let bonds_result = rpc.call("bonds.list", serde_json::json!({})).await?;
    let existing = bonds_result["bonds"].as_array().cloned().unwrap_or_default();
    println!("  found {}.", existing.len());

    // -- Call AI via MCP server ----------------------------------------------

    print!("  {} Asking AI to analyse your goal…", "→".cyan());
    let assistance = mcp.bond_assist(&description, &existing).await?;
    println!("  done.\n");

    // -- Display analysis ----------------------------------------------------

    println!("{}", "  ── Analysis ──────────────────────────────────────────".dimmed());
    println!("  {}\n", assistance.analysis);

    // -- Display similar bonds -----------------------------------------------

    if assistance.similar_bonds.is_empty() {
        println!("  {}", "No similar bonds found on this network.".dimmed());
    } else {
        println!("{}", "  ── Similar existing bonds ────────────────────────────".dimmed());
        for (i, b) in assistance.similar_bonds.iter().enumerate() {
            let sim_color = match b.similarity.as_str() {
                "high"   => b.similarity.green(),
                "medium" => b.similarity.yellow(),
                _        => b.similarity.dimmed(),
            };
            println!("  [{}] {}  ({})", i + 1, b.title.bold(), sim_color);
            println!("      ID: {}", b.bond_id.dimmed());
            println!("      {}\n", b.explanation);
        }
    }

    // -- Display AI-drafted bond ---------------------------------------------

    let s = &assistance.suggested_bond;
    println!("{}", "  ── AI-drafted bond ───────────────────────────────────".dimmed());
    println!("  Title:    {}", s.title.bold());
    println!("  Goal:     {}", s.description);
    println!("  Data ID:  {}   {} {}", s.data_id.cyan(), s.operator, s.threshold);
    if let Some(a) = &s.aggregation     { println!("  Aggreg.:  {a}"); }
    println!("  Deadline: {}", s.deadline);
    if let Some(n) = &s.notes {
        println!("\n  {} {}", "Note:".yellow().bold(), n.yellow());
    }
    println!();

    // -- User choice ---------------------------------------------------------

    let mut choices = vec!["Create bond from AI draft".to_owned()];
    for b in &assistance.similar_bonds {
        choices.push(format!("Back existing bond: {}", b.title));
    }
    choices.push("Start fresh (ignore AI suggestions)".to_owned());
    choices.push("Cancel".to_owned());

    let choice_idx = Select::with_theme(&theme)
        .with_prompt("What would you like to do?")
        .items(&choices)
        .default(0)
        .interact()?;

    let n_similar = assistance.similar_bonds.len();

    if choice_idx == 0 {
        // Use AI draft.
        let defaults = BondDefaults {
            title:           s.title.clone(),
            description:     s.description.clone(),
            data_id:         s.data_id.clone(),
            operator_idx:    s.operator_idx(),
            threshold:       s.threshold.clone(),
            aggregation:     s.aggregation.clone().unwrap_or_default(),
            evidence_format: s.evidence_format.clone().unwrap_or_default(),
            deadline:        s.deadline.clone(),
        };
        println!("\n{}", "  NyxForge — Propose Bond (AI-assisted)".bold().purple());
        let mut bond = run_wizard(rpc, Some(&defaults)).await?;
        bond.state = BondState::Proposed;

        println!("\n{}", "  --- Proposal preview ---".bold());
        println!("{}", serde_json::to_string_pretty(&bond)?);
        println!();
        if !Confirm::with_theme(&theme)
            .with_prompt("Publish this proposal?")
            .default(true)
            .interact()?
        {
            println!("{}", "  Cancelled.".dimmed());
            return Ok(());
        }
        let result = rpc.call("bonds.propose", serde_json::json!({ "bond": bond })).await?;
        let bond_id = result["bond_id"].as_str().unwrap_or("(unknown)");
        println!("\n  {} Proposal published: {}", "✔".green().bold(), bond_id.cyan().bold());
        println!("  {}", "Community members can now post questions with: bond comment <id>".dimmed());

    } else if choice_idx <= n_similar {
        // Back an existing bond.
        let selected = &assistance.similar_bonds[choice_idx - 1];
        println!("\n  {} Selected: {}", "→".cyan(), selected.title.bold());
        println!("  Bond ID: {}", selected.bond_id.cyan());
        println!();
        // Show full bond details.
        let info = rpc.call("bonds.get", serde_json::json!({ "id": selected.bond_id })).await?;
        println!("{}", serde_json::to_string_pretty(&info)?);
        println!();
        println!("  {}", "To buy this bond, find it in the market UI or ask a peer for its current ask price.".dimmed());

    } else if choice_idx == n_similar + 1 {
        // Start fresh.
        println!("\n{}", "  NyxForge — Propose Bond".bold().purple());
        let mut bond = run_wizard(rpc, None).await?;
        bond.state = BondState::Proposed;

        println!("\n{}", "  --- Proposal preview ---".bold());
        println!("{}", serde_json::to_string_pretty(&bond)?);
        println!();
        if !Confirm::with_theme(&theme)
            .with_prompt("Publish this proposal?")
            .default(false)
            .interact()?
        {
            println!("{}", "  Cancelled.".dimmed());
            return Ok(());
        }
        let result = rpc.call("bonds.propose", serde_json::json!({ "bond": bond })).await?;
        let bond_id = result["bond_id"].as_str().unwrap_or("(unknown)");
        println!("\n  {} Proposal published: {}", "✔".green().bold(), bond_id.cyan().bold());

    } else {
        println!("{}", "  Cancelled.".dimmed());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// bond propose
// ---------------------------------------------------------------------------

async fn cmd_propose(rpc: &RpcClient) -> Result<()> {
    println!("\n{}", "  NyxForge — Propose Bond".bold().purple());
    println!("  {}", "Your proposal will be published for community review.".dimmed());
    println!("  {}\n", "Others can post questions and suggestions before you issue it.".dimmed());

    let mut bond = run_wizard(rpc, None).await?;
    bond.state = BondState::Proposed;

    println!("\n{}", "  --- Proposal preview ---".bold());
    println!("{}", serde_json::to_string_pretty(&bond)?);

    println!();
    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Publish this proposal?")
        .default(false)
        .interact()?
    {
        println!("{}", "  Cancelled.".dimmed());
        return Ok(());
    }

    let result = rpc.call("bonds.propose", serde_json::json!({ "bond": bond })).await?;
    let bond_id = result["bond_id"].as_str().unwrap_or("(unknown)");
    println!("\n  {} Proposal published: {}", "✔".green().bold(), bond_id.cyan().bold());
    println!("  {}", "Community members can now post questions with: bond comment <id>".dimmed());
    Ok(())
}

// ---------------------------------------------------------------------------
// bond create — interactive wizard
// ---------------------------------------------------------------------------

async fn cmd_create(rpc: &RpcClient) -> Result<()> {
    println!("\n{}", "  NyxForge — Create Bond".bold().purple());
    println!("  {}\n", "Bond will be submitted for oracle approval before going live.".dimmed());

    let bond = run_wizard(rpc, None).await?;

    println!("\n{}", "  --- Bond preview ---".bold());
    println!("{}", serde_json::to_string_pretty(&bond)?);

    println!();
    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Submit for oracle approval?")
        .default(false)
        .interact()?
    {
        println!("{}", "  Cancelled.".dimmed());
        return Ok(());
    }

    let result = rpc.call("bonds.submit_for_approval", serde_json::json!({ "bond": bond })).await?;
    let bond_id = result["bond_id"].as_str().unwrap_or("(unknown)");
    let awaiting = result["awaiting"].as_u64().unwrap_or(0);
    println!("\n  {} Submitted for oracle approval: {}", "✔".green().bold(), bond_id.cyan().bold());
    println!("  {} oracle(s) must accept before the bond goes live.", awaiting);
    println!("  Check status with: {} {}", "bond oracle-status".dimmed(), bond_id.dimmed());
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
        let title = b["goals"][0]["title"].as_str().unwrap_or("?");
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
// bond submit — submit a bond for oracle approval
// ---------------------------------------------------------------------------

async fn cmd_submit(rpc: &RpcClient, existing_id: Option<&str>) -> Result<()> {
    let theme = ColorfulTheme::default();
    let params = if let Some(id) = existing_id {
        // Transition an existing Proposed bond.
        let info = rpc.call("bonds.get", serde_json::json!({ "id": id })).await?;
        let state = info["state"].as_str().unwrap_or("?");
        if state != "Proposed" {
            return Err(anyhow!("Bond is in '{}' state; only Proposed bonds can be submitted for approval.", state));
        }
        let title = info["goals"][0]["title"].as_str().unwrap_or("(unknown)");
        println!("\n  Submitting for oracle approval: {}", title.bold());
        if !Confirm::with_theme(&theme)
            .with_prompt("Submit this proposal for oracle approval?")
            .default(true)
            .interact()? {
            println!("{}", "  Cancelled.".dimmed());
            return Ok(());
        }
        serde_json::json!({ "bond_id": id })
    } else {
        // Run the wizard and submit directly.
        println!("\n{}", "  NyxForge — Submit Bond for Oracle Approval".bold().purple());
        let bond = run_wizard(rpc, None).await?;
        println!("\n{}", "  --- Bond preview ---".bold());
        println!("{}", serde_json::to_string_pretty(&bond)?);
        println!();
        if !Confirm::with_theme(&theme)
            .with_prompt("Submit for oracle approval?")
            .default(false)
            .interact()? {
            println!("{}", "  Cancelled.".dimmed());
            return Ok(());
        }
        serde_json::json!({ "bond": bond })
    };

    let result = rpc.call("bonds.submit_for_approval", params).await?;
    let bond_id = result["bond_id"].as_str().unwrap_or("(unknown)");
    let awaiting = result["awaiting"].as_u64().unwrap_or(0);
    println!("\n  {} Submitted: {}", "✔".green().bold(), bond_id.cyan().bold());
    println!("  {} oracle(s) must accept. Check status: {} {}",
        awaiting, "bond oracle-status".dimmed(), bond_id.dimmed());
    Ok(())
}

// ---------------------------------------------------------------------------
// bond issue — lock collateral and activate a Draft bond
// ---------------------------------------------------------------------------

async fn cmd_issue(rpc: &RpcClient, bond_id: &str) -> Result<()> {
    let theme = ColorfulTheme::default();
    // Show current state and oracle status before confirming.
    let info = rpc.call("bonds.get", serde_json::json!({ "id": bond_id })).await?;
    let state = info["state"].as_str().unwrap_or("?");
    if state != "Draft" {
        return Err(anyhow!(
            "Bond is in '{}' state. It must be in Draft state (all oracles accepted) before issuance.",
            state
        ));
    }
    let title = info["goals"][0]["title"].as_str().unwrap_or("(unknown)");
    println!("\n  Issuing bond: {}", title.bold());
    println!("  All oracles have accepted. This will lock collateral and go live.");
    if !Confirm::with_theme(&theme)
        .with_prompt("Lock collateral and issue bond?")
        .default(false)
        .interact()? {
        println!("{}", "  Cancelled.".dimmed());
        return Ok(());
    }
    let result = rpc.call("bonds.issue", serde_json::json!({ "bond_id": bond_id })).await?;
    let id = result["bond_id"].as_str().unwrap_or("(unknown)");
    println!("\n  {} Bond live: {}", "✔".green().bold(), id.cyan().bold());
    Ok(())
}

// ---------------------------------------------------------------------------
// bond oracle-accept / oracle-reject / oracle-status / revise-oracles
// ---------------------------------------------------------------------------

async fn cmd_oracle_accept(rpc: &RpcClient, bond_id: &str) -> Result<()> {
    let info = rpc.call("bonds.get", serde_json::json!({ "id": bond_id })).await?;
    let title = info["goals"][0]["title"].as_str().unwrap_or("(unknown)");
    println!("\n  Accepting oracle responsibility for: {}", title.bold());

    let oracle_key = fetch_drk_pubkey(rpc)
        .await
        .ok_or_else(|| anyhow!("No wallet found. Create one first with: nyxforge-cli wallet create"))?;

    let result = rpc.call("bonds.oracle_accept", serde_json::json!({
        "bond_id":    bond_id,
        "oracle_key": oracle_key,
    })).await?;

    let new_state = result["bond_state"].as_str().unwrap_or("?");
    println!("\n  {} Accepted. Bond state: {}", "✔".green().bold(), new_state.cyan());
    if let Some(pending) = result["still_pending"].as_array() {
        if !pending.is_empty() {
            println!("  Still waiting on {} oracle(s).", pending.len());
        }
    }
    if let Some(msg) = result["message"].as_str() {
        println!("  {}", msg.dimmed());
    }
    Ok(())
}

async fn cmd_oracle_reject(rpc: &RpcClient, bond_id: &str) -> Result<()> {
    let theme = ColorfulTheme::default();
    let info = rpc.call("bonds.get", serde_json::json!({ "id": bond_id })).await?;
    let title = info["goals"][0]["title"].as_str().unwrap_or("(unknown)");
    println!("\n  Rejecting oracle responsibility for: {}", title.bold());

    let oracle_key = fetch_drk_pubkey(rpc)
        .await
        .ok_or_else(|| anyhow!("No wallet found. Create one first with: nyxforge-cli wallet create"))?;

    let reason: String = Input::with_theme(&theme)
        .with_prompt("Reason for rejection (required)")
        .interact_text()?;
    if reason.trim().is_empty() {
        return Err(anyhow!("A reason is required when rejecting a bond."));
    }

    let result = rpc.call("bonds.oracle_reject", serde_json::json!({
        "bond_id":    bond_id,
        "oracle_key": oracle_key,
        "reason":     reason,
    })).await?;

    println!("\n  {} Rejection recorded.", "✔".green().bold());
    if let Some(msg) = result["message"].as_str() {
        println!("  {}", msg.dimmed());
    }
    Ok(())
}

async fn cmd_oracle_status(rpc: &RpcClient, bond_id: &str) -> Result<()> {
    let result = rpc.call("bonds.oracle_status", serde_json::json!({ "bond_id": bond_id })).await?;
    let bond_state = result["bond_state"].as_str().unwrap_or("?");
    println!("\n  Bond state: {}\n", bond_state.cyan().bold());

    let oracles = result["oracles"].as_array()
        .ok_or_else(|| anyhow!("unexpected response"))?;

    for o in oracles {
        let key    = o["oracle"].as_str().unwrap_or("?");
        let status = o["status"].as_str().unwrap_or("?");
        let ts     = o["responded_at"].as_str().unwrap_or("");
        let reason = o["reason"].as_str().unwrap_or("");

        let status_str = match status {
            "accepted" => format!("{}", "✔ accepted".green()),
            "rejected" => format!("{}", "✘ rejected".red()),
            _          => format!("{}", "… pending".yellow()),
        };
        print!("  {}…  {}", &key[..8.min(key.len())].cyan(), status_str);
        if !ts.is_empty() { print!("  {}", ts.dimmed()); }
        println!();
        if !reason.is_empty() { println!("    Reason: {}", reason.yellow()); }
    }
    println!();
    Ok(())
}

async fn cmd_revise_oracles(rpc: &RpcClient, bond_id: &str) -> Result<()> {
    let theme = ColorfulTheme::default();
    println!("\n  {}", "Replace oracle list — all existing responses will be cleared.".yellow());

    let default_key = fetch_drk_pubkey(rpc).await;
    let prompt = match &default_key {
        Some(k) => format!("New oracle keys (hex, comma-separated) [default: your key {}]", &k[..8]),
        None    => "New oracle keys (hex, comma-separated)".into(),
    };
    let keys_raw: String = Input::with_theme(&theme)
        .with_prompt(prompt)
        .allow_empty(default_key.is_some())
        .interact_text()?;

    let keys: Vec<String> = if keys_raw.trim().is_empty() {
        match default_key {
            Some(k) => vec![k],
            None    => return Err(anyhow!("No oracle key provided and no wallet found.")),
        }
    } else {
        keys_raw.split(',').map(|s| s.trim().to_owned()).collect()
    };

    if !Confirm::with_theme(&theme)
        .with_prompt(format!("Replace oracles with {} key(s) and clear all responses?", keys.len()))
        .default(false)
        .interact()? {
        println!("{}", "  Cancelled.".dimmed());
        return Ok(());
    }

    let result = rpc.call("bonds.revise_oracles", serde_json::json!({
        "bond_id":     bond_id,
        "oracle_keys": keys,
    })).await?;

    println!("\n  {} {}", "✔".green().bold(),
        result["message"].as_str().unwrap_or("Oracle list updated.").dimmed());
    println!("  Check new status: {} {}", "bond oracle-status".dimmed(), bond_id.dimmed());
    Ok(())
}

// ---------------------------------------------------------------------------
// bond comment
// ---------------------------------------------------------------------------

async fn cmd_comment(rpc: &RpcClient, bond_id: &str) -> Result<()> {
    let theme = ColorfulTheme::default();

    // Verify the bond exists and is in Proposed state.
    let info = rpc.call("bonds.get", serde_json::json!({ "id": bond_id })).await?;
    let state = info["state"].as_str().unwrap_or("?");
    if state != "Proposed" {
        return Err(anyhow!("Bond is in '{}' state. Comments are only accepted on Proposed bonds.", state));
    }
    let title = info["goals"][0]["title"].as_str().unwrap_or("(unknown)");
    println!("\n  Commenting on: {}", title.bold());

    let body: String = Input::with_theme(&theme)
        .with_prompt("Your question or suggestion")
        .interact_text()?;
    if body.trim().is_empty() {
        println!("{}", "  Cancelled — empty comment.".dimmed());
        return Ok(());
    }

    let author_hex = fetch_drk_pubkey(rpc)
        .await
        .ok_or_else(|| anyhow!("No wallet found. Create one first with: nyxforge-cli wallet create"))?;

    let result = rpc.call("bonds.comment", serde_json::json!({
        "bond_id": bond_id,
        "author":  author_hex,
        "body":    body,
    })).await?;
    let comment_id = result["comment_id"].as_str().unwrap_or("(unknown)");
    println!("\n  {} Comment posted: {}", "✔".green().bold(), comment_id.cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// bond comments
// ---------------------------------------------------------------------------

async fn cmd_comments(rpc: &RpcClient, bond_id: &str) -> Result<()> {
    let result = rpc.call("bonds.comments", serde_json::json!({ "bond_id": bond_id })).await?;
    let comments = result["comments"]
        .as_array()
        .ok_or_else(|| anyhow!("unexpected response from bonds.comments"))?;

    if comments.is_empty() {
        println!("{}", "  No comments yet.".dimmed());
        return Ok(());
    }

    println!("\n  {} comment(s)\n", comments.len());
    for (i, c) in comments.iter().enumerate() {
        let author = c["author"].as_object()
            .and_then(|o| o.values().next())
            .and_then(|v| v.as_array())
            .map(|arr| {
                let bytes: Vec<u8> = arr.iter().filter_map(|x| x.as_u64()).map(|x| x as u8).collect();
                hex::encode(&bytes[..4.min(bytes.len())])
            })
            .unwrap_or_else(|| "?".into());
        let ts = c["created_at"].as_str().unwrap_or("?");
        let body = c["body"].as_str().unwrap_or("?");
        println!("  {}  {}…  {}", format!("[{}]", i + 1).dimmed(), author.cyan(), ts.dimmed());
        println!("  {}\n", body);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// bond auction-price
// ---------------------------------------------------------------------------

async fn cmd_auction_price(rpc: &RpcClient, bond_id: &str) -> Result<()> {
    let result = rpc.call("bonds.auction_price", serde_json::json!({ "bond_id": bond_id })).await?;
    let price = result["price_micro_drk"].as_u64().unwrap_or(0);
    let drk = price as f64 / 1_000_000.0;
    println!("\n  Current auction price: {} DRK  ({} micro-DRK)", drk, price);
    Ok(())
}

// ---------------------------------------------------------------------------
// bond buy
// ---------------------------------------------------------------------------

async fn cmd_buy(rpc: &RpcClient, bond_id: &str, quantity: u64) -> Result<()> {
    let result = rpc.call("bonds.buy", serde_json::json!({
        "bond_id":  bond_id,
        "quantity": quantity,
    })).await?;
    let purchased = result["purchased"].as_u64().unwrap_or(0);
    let price     = result["price_micro_drk"].as_u64().unwrap_or(0);
    let drk = price as f64 / 1_000_000.0;
    println!("\n  {} Purchased {} bond(s) at {} DRK each.", "✔".green().bold(), purchased, drk);
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
