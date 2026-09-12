use anyhow::{anyhow, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use rust_decimal::Decimal;
use std::path::PathBuf;
use std::str::FromStr;

use crate::mcp_client::McpClient;
use crate::rpc_client::RpcClient;

// ---------------------------------------------------------------------------
// AI wizard defaults (reserved for future `bounty explore` command)
// ---------------------------------------------------------------------------

pub struct BountyDefaults {
    pub title:        String,
    pub description:  String,
    pub deadline:     String,  // YYYY-MM-DD
    pub expiry:       String,  // YYYY-MM-DD
    // Per-term defaults added when `bounty explore` is implemented; see Term struct.
}

// ---------------------------------------------------------------------------
// Wizard output types
// ---------------------------------------------------------------------------

struct Term {
    goal_type:   String,          // "quantitative" | "qualitative" | "hybrid"
    criterion:   String,          // human-readable description of the condition
    data_id:     Option<String>,
    operator:    Option<String>,  // "lt" | "lte" | "gt" | "gte" | "eq"
    threshold:   Option<String>,
    aggregation: Option<String>,
}

struct OracleEntry {
    pubkey:  String,
    role:    String,   // "quantitative" | "qualitative" | "both"
    fee_xmr: String,  // decimal
}

struct WizardSpec {
    title:            String,
    description:      String,
    terms:            Vec<Term>,
    term_aggregation: Option<String>,  // "AND" | "OR"; None when bounty has only one term
    deadline:         String,          // YYYY-MM-DD
    expiry:           String,          // YYYY-MM-DD (must be after deadline)
    grace_days:       u32,             // days after expiry before timelock matures
    currency:         String,          // "xmr" | "zec" | "btc" | "eth"
    lock_mechanism:   String,          // "dleq_xmr" | "dleq_zec_sapling" | "ptlc_btc" | "eth_escrow"
    chain_id:         Option<u32>,     // ETH only: 1=mainnet 11155111=Sepolia 17000=Holesky
    amount:           String,          // decimal, per file; units are in `currency`
    redemption_value: Option<String>,  // fiat peg at issuance, informational (e.g. "1000 USD")
    unit_count:       u64,
    alg_epoch:        u32,             // 0 = classical
    key_algorithm:    String,          // "ed25519" | "jubjub" | "secp256k1"
    enc_algorithm:    String,          // "x25519-chacha20poly1305"
    // Issue-time fields -- null in DRAFT; set during bounty issue
    t_met:            Option<String>,
    t_fail:           Option<String>,
    lock_address:     Option<String>,
    oracle_eth_pubkey: Option<String>,
    holder_pubkey:    Option<String>,
    scalar_encrypted: Option<String>,
    // PQ slots -- null until v3 migration
    pq_holder_pubkey: Option<String>,
    pq_scalar_enc:    Option<String>,
    judges:          Vec<OracleEntry>,
    quorum:           u32,
    challenge_days:   u32,
    output_dir:       PathBuf,
}

// ---------------------------------------------------------------------------
// Clap types
// ---------------------------------------------------------------------------

#[derive(Parser)]
pub struct BountyArgs {
    #[command(subcommand)]
    pub command: BountyCommand,
}

#[derive(Subcommand)]
pub enum BountyCommand {
    /// Interactive wizard -- outputs one or more .bounty SQLite files
    Create,

    /// Display the contents of a .bounty file
    Inspect {
        /// Path to the .bounty file
        file: PathBuf,
        /// BountyViewKey hex (required for SQLCipher-encrypted files)
        #[arg(long)]
        key: Option<String>,
    },

    /// Show current state, attestation count, and deadline
    Status {
        /// Path to the .bounty file
        file: PathBuf,
    },

    /// List .bounty files in a directory
    List {
        /// Directory to scan (default: current directory)
        #[arg(default_value = ".")]
        dir: PathBuf,
    },

    /// Re-encrypt the secret scalar to a new holder
    Transfer {
        /// Path to the .bounty file
        file: PathBuf,
        /// Recipient public key (32-byte hex)
        recipient: String,
    },

    /// Run the WASM verifier against bundled evidence
    Verify {
        /// Path to the .bounty file
        file: PathBuf,
    },

    /// Lock XMR collateral on-chain and activate a DRAFT bounty
    Issue {
        /// Path to the .bounty file
        file: PathBuf,
    },

    /// Construct and broadcast the XMR sweep using adaptor + s_met
    Redeem {
        /// Path to the .bounty file
        file: PathBuf,
    },

    /// Sweep collateral back to issuer after expiry (s_fail path)
    Reclaim {
        /// Path to the .bounty file
        file: PathBuf,
    },

    /// Generate a signed listing record proving ownership (for exchanges)
    Prove {
        /// Path to the .bounty file
        file: PathBuf,
        /// Challenge nonce from the exchange (hex)
        #[arg(long)]
        nonce: Option<String>,
        /// Redemption price ask (collateral units, e.g. 1.2)
        #[arg(long)]
        ask: Option<String>,
        /// Listing record expiry date (YYYY-MM-DD)
        #[arg(long, name = "list-expiry")]
        list_expiry: Option<String>,
    },

    /// Developer and stagenet testing utilities
    Dev {
        #[command(subcommand)]
        action: DevAction,
    },
}

#[derive(Subcommand)]
pub enum DevAction {
    /// Mock an judge attestation for stagenet testing
    MockAttest {
        /// Bounty ID (hex)
        id: String,
        /// Attested goal result
        goal_met: bool,
    },
    /// Force a bounty state transition (bypasses normal checks)
    ForceState {
        /// Bounty ID (hex)
        id: String,
        /// Target state (Draft, Active, Redeemable, Settled, Expired, Reclaimed)
        state: String,
    },
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub async fn run(args: BountyArgs, rpc: &RpcClient, _mcp: &McpClient) -> Result<()> {
    match args.command {
        BountyCommand::Create                       => cmd_create().await,
        BountyCommand::Inspect { file, key }        => cmd_inspect(&file, key.as_deref()).await,
        BountyCommand::Status { file }              => cmd_status(&file).await,
        BountyCommand::List { dir }                 => cmd_list(&dir).await,
        BountyCommand::Transfer { file, recipient } => cmd_transfer(&file, &recipient).await,
        BountyCommand::Verify { file }              => cmd_verify(&file).await,
        BountyCommand::Issue { file }               => cmd_issue(&file).await,
        BountyCommand::Redeem { file }              => cmd_redeem(&file).await,
        BountyCommand::Reclaim { file }             => cmd_reclaim(&file).await,
        BountyCommand::Prove { file, nonce, ask, list_expiry }
                                                  => cmd_prove(&file, nonce.as_deref(), ask.as_deref(), list_expiry.as_deref()).await,
        BountyCommand::Dev { action }               => cmd_dev(rpc, action).await,
    }
}

// ---------------------------------------------------------------------------
// bounty create
// ---------------------------------------------------------------------------

async fn cmd_create() -> Result<()> {
    let theme = ColorfulTheme::default();
    println!("\n{}", "  NyxForge -- Create Bounty".bold().purple());
    println!("  {}\n", "Wizard writes one or more .bounty SQLite files to the output directory.".dimmed());

    let spec = run_wizard(None)?;

    println!("\n{}", "  -- Preview --".bold());
    print_spec_preview(&spec);
    println!();

    if !Confirm::with_theme(&theme)
        .with_prompt("Write .bounty file(s)?")
        .default(false)
        .interact()?
    {
        println!("{}", "  Cancelled.".dimmed());
        return Ok(());
    }

    // TODO(phase-1): nyxforge_bounty::create_series(&spec, &spec.output_dir)
    //   Writes <series_id>-<serial_4digits>.bounty files via rusqlite.
    //   Each file gets a fresh secret scalar and adaptor commitment.
    println!("  {} .bounty file creation requires the nyxforge-bounty crate (Phase 1).", "!".yellow());
    Ok(())
}

// ---------------------------------------------------------------------------
// bounty inspect
// ---------------------------------------------------------------------------

async fn cmd_inspect(file: &PathBuf, _key: Option<&str>) -> Result<()> {
    require_file(file)?;
    // TODO(phase-1): nyxforge_bounty::inspect(file, key)
    //   Prints all tables: bounty_spec, collateral, judges, oracle_config,
    //   evidence (summary), attestations, history, verifier metadata.
    println!("  {} bounty inspect requires the nyxforge-bounty crate (Phase 1).", "!".yellow());
    println!("  File: {}", file.display().to_string().cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// bounty status
// ---------------------------------------------------------------------------

async fn cmd_status(file: &PathBuf) -> Result<()> {
    require_file(file)?;
    // TODO(phase-1): read history, attestations, bounty_spec deadline, judge quorum.
    //   Print: current state, attestations received/required, days to deadline.
    println!("  {} bounty status requires the nyxforge-bounty crate (Phase 1).", "!".yellow());
    println!("  File: {}", file.display().to_string().cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// bounty list
// ---------------------------------------------------------------------------

async fn cmd_list(dir: &PathBuf) -> Result<()> {
    if !dir.is_dir() {
        return Err(anyhow!("Not a directory: {}", dir.display()));
    }
    let mut count = 0u64;
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("bounty"))
        .collect();
    entries.sort();
    for path in &entries {
        println!("  {}", path.display().to_string().cyan());
        count += 1;
    }
    if count == 0 {
        println!("{}", "  No .bounty files found.".dimmed());
    } else {
        println!("\n  {} file(s)", count);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// bounty transfer
// ---------------------------------------------------------------------------

async fn cmd_transfer(file: &PathBuf, recipient: &str) -> Result<()> {
    require_file(file)?;
    validate_hex_key(recipient)?;
    // TODO(phase-2): nyxforge_bounty::transfer(file, recipient_pubkey)
    //   Decrypts secret scalar with current holder key, re-encrypts to recipient,
    //   writes updated .bounty file. Prints new file path.
    println!("  {} bounty transfer requires the nyxforge-bounty crate (Phase 2).", "!".yellow());
    println!("  File:      {}", file.display().to_string().cyan());
    println!("  Recipient: {}...", &recipient[..8.min(recipient.len())].cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// bounty verify
// ---------------------------------------------------------------------------

async fn cmd_verify(file: &PathBuf) -> Result<()> {
    require_file(file)?;
    // TODO(phase-2): load wasm_blob from verifier table, instantiate with wasmtime,
    //   pass evidence BLOBs as input, print pass/fail and any verifier output.
    println!("  {} bounty verify requires WASM execution support (Phase 2).", "!".yellow());
    println!("  File: {}", file.display().to_string().cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// bounty issue
// ---------------------------------------------------------------------------

async fn cmd_issue(file: &PathBuf) -> Result<()> {
    require_file(file)?;
    // TODO(phase-4): read collateral table, construct DLEQ commitment (s_met/s_fail
    //   scalars), build and broadcast XMR lock transaction, write lock_txid back to
    //   collateral table, write ACTIVE state to history table.
    println!("  {} bounty issue requires DLEQ setup and XMR broadcast (Phase 4).", "!".yellow());
    println!("  File: {}", file.display().to_string().cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// bounty redeem
// ---------------------------------------------------------------------------

async fn cmd_redeem(file: &PathBuf) -> Result<()> {
    require_file(file)?;
    // TODO(phase-4): verify bounty is REDEEMABLE, load adaptor BLOB from collateral
    //   table, combine with judge-published s_met, construct complete XMR
    //   transaction, broadcast to Monero network, write SETTLED to history.
    println!("  {} bounty redeem requires DLEQ + XMR sweep logic (Phase 4).", "!".yellow());
    println!("  File: {}", file.display().to_string().cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// bounty reclaim
// ---------------------------------------------------------------------------

async fn cmd_reclaim(file: &PathBuf) -> Result<()> {
    require_file(file)?;
    // TODO(phase-4): verify expiry date has passed, retrieve judge s_fail scalar,
    //   construct and broadcast XMR sweep to issuer return address, write RECLAIMED
    //   to history table.
    println!("  {} bounty reclaim requires DLEQ + XMR sweep logic (Phase 4).", "!".yellow());
    println!("  File: {}", file.display().to_string().cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// bounty prove
// ---------------------------------------------------------------------------

async fn cmd_prove(
    file:        &PathBuf,
    _nonce:      Option<&str>,
    _ask:        Option<&str>,
    _list_expiry: Option<&str>,
) -> Result<()> {
    require_file(file)?;
    // TODO(phase-2): load holder_pubkey from collateral table, sign
    //   Blake3(nonce || bounty_id || ask || list_expiry) with holder private key,
    //   emit listing record JSON to stdout.
    println!("  {} bounty prove requires the nyxforge-bounty crate (Phase 2).", "!".yellow());
    println!("  File: {}", file.display().to_string().cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// dev commands
// ---------------------------------------------------------------------------

async fn cmd_dev(rpc: &RpcClient, action: DevAction) -> Result<()> {
    match action {
        DevAction::MockAttest { id, goal_met } => {
            println!("\n  {} Mocking attestation for bounty {}...", "->".cyan(), id);
            let result = rpc.call("dev.mock_attest", serde_json::json!({
                "bounty_id":  id,
                "goal_met": goal_met,
            })).await?;
            println!("  {} {}", "OK".green(), result["message"].as_str().unwrap_or("Done."));
        }
        DevAction::ForceState { id, state } => {
            println!("\n  {} Forcing state '{}' for bounty {}...", "->".cyan(), state, id);
            let result = rpc.call("dev.force_state", serde_json::json!({
                "bounty_id": id,
                "state":   state,
            })).await?;
            println!("  {} {}", "OK".green(), result["message"].as_str().unwrap_or("State updated."));
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Wizard
// ---------------------------------------------------------------------------

fn lock_mechanism_for_currency(currency: &str) -> String {
    match currency {
        "zec" => "dleq_zec_sapling",
        "btc" => "ptlc_btc",
        "eth" => "eth_escrow",
        _     => "dleq_xmr",
    }.to_owned()
}

fn key_algorithm_for_currency(currency: &str) -> String {
    match currency {
        "zec" => "jubjub",
        "btc" | "eth" => "secp256k1",
        _ => "ed25519",
    }.to_owned()
}

fn run_wizard(defaults: Option<&BountyDefaults>) -> Result<WizardSpec> {
    let theme = ColorfulTheme::default();

    if defaults.is_some() {
        println!("  {}\n", "AI suggestions are pre-filled -- edit freely, then press Enter.".dimmed());
    } else {
        println!("  {}\n", "Answer each prompt.  Press Enter to accept a default.".dimmed());
    }

    // -- Bounty identity -------------------------------------------------------

    println!("  {}", "Bounty identity".bold());

    let mut title_input = Input::<String>::with_theme(&theme).with_prompt("Title");
    if let Some(d) = defaults { title_input = title_input.with_initial_text(&d.title); }
    let title = title_input.interact_text()?;

    let mut desc_input = Input::<String>::with_theme(&theme).with_prompt("Description");
    if let Some(d) = defaults { desc_input = desc_input.with_initial_text(&d.description); }
    let description = desc_input.interact_text()?;

    // -- Terms ---------------------------------------------------------------

    println!("\n  {}", "Terms".bold());
    println!("  {}\n", "Each term is one measurable condition. The bounty pays out based on the aggregation of all terms.".dimmed());

    let mut terms: Vec<Term> = Vec::new();

    loop {
        let term_num = terms.len() + 1;
        println!("  {}", format!("Term {}", term_num).bold());

        let goal_type_labels = &["quantitative", "qualitative", "hybrid"];
        let goal_type_idx = Select::with_theme(&theme)
            .with_prompt("  Goal type")
            .items(goal_type_labels)
            .default(0)
            .interact()?;
        let term_goal_type = goal_type_labels[goal_type_idx].to_owned();

        let criterion: String = Input::with_theme(&theme)
            .with_prompt("  Criterion (human-readable description)")
            .interact_text()?;

        let (data_id, operator, threshold, aggregation) = if term_goal_type != "qualitative" {
            let data_id_s: String = Input::with_theme(&theme)
                .with_prompt("  Data ID (e.g. usgov.cbo.federal_outlays_pct_gdp; Enter to skip)")
                .allow_empty(true)
                .interact_text()?;
            let data_id = if data_id_s.is_empty() { None } else { Some(data_id_s) };

            let op_labels = &["lt", "lte", "gt", "gte", "eq"];
            let op_idx = Select::with_theme(&theme)
                .with_prompt("  Operator")
                .items(op_labels)
                .default(3) // default: gte
                .interact()?;
            let operator = op_labels[op_idx].to_owned();

            let threshold: String = loop {
                let s: String = Input::with_theme(&theme)
                    .with_prompt("  Threshold value")
                    .interact_text()?;
                if Decimal::from_str(&s).is_ok() { break s; }
                eprintln!("  {} Enter a valid decimal number.", "!".red());
            };

            let agg_s: String = Input::with_theme(&theme)
                .with_prompt("  Aggregation (e.g. annual_mean; Enter to skip)")
                .allow_empty(true)
                .interact_text()?;

            (data_id, Some(operator), Some(threshold),
             if agg_s.is_empty() { None } else { Some(agg_s) })
        } else {
            (None, None, None, None)
        };

        terms.push(Term { goal_type: term_goal_type, criterion, data_id, operator, threshold, aggregation });

        let add_more = Confirm::with_theme(&theme)
            .with_prompt("Add another term?")
            .default(false)
            .interact()?;
        if !add_more { break; }
    }

    let term_aggregation: Option<String> = if terms.len() > 1 {
        let agg_labels = &[
            "AND -- all terms must be met",
            "OR  -- any term must be met",
        ];
        let agg_keys = &["AND", "OR"];
        let agg_idx = Select::with_theme(&theme)
            .with_prompt("Aggregation across terms")
            .items(agg_labels)
            .default(0)
            .interact()?;
        Some(agg_keys[agg_idx].to_owned())
    } else {
        None
    };

    // -- Timing --------------------------------------------------------------

    println!("\n  {}", "Timing".bold());

    let deadline = loop {
        let mut dl_input = Input::<String>::with_theme(&theme)
            .with_prompt("Goal deadline (YYYY-MM-DD, max 10 years from today)");
        if let Some(d) = defaults { dl_input = dl_input.with_initial_text(&d.deadline); }
        let s = dl_input.interact_text()?;
        match NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            Ok(dl) => {
                let today = Local::now().date_naive();
                let max_year  = today.year() + 10;
                let max_deadline = NaiveDate::from_ymd_opt(max_year, today.month(), today.day())
                    .unwrap_or(NaiveDate::from_ymd_opt(max_year, 12, 31).unwrap());
                if dl > max_deadline {
                    eprintln!("  {} Deadline must be within 10 years of today (max {}).", "!".red(), max_deadline);
                } else {
                    break s;
                }
            }
            Err(_) => eprintln!("  {} Use YYYY-MM-DD format.", "!".red()),
        }
    };

    let expiry = loop {
        let mut ex_input = Input::<String>::with_theme(&theme)
            .with_prompt("Hard expiry -- judge settlement window ends (YYYY-MM-DD, after deadline)");
        if let Some(d) = defaults { ex_input = ex_input.with_initial_text(&d.expiry); }
        let s = ex_input.interact_text()?;
        match NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
            Ok(exp) => {
                let dl = NaiveDate::parse_from_str(&deadline, "%Y-%m-%d").unwrap();
                if exp > dl { break s; }
                eprintln!("  {} Expiry must be after deadline ({}).", "!".red(), deadline);
            }
            Err(_) => eprintln!("  {} Use YYYY-MM-DD format.", "!".red()),
        }
    };

    let grace_days: u32 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Grace period (days after expiry before timelock matures)")
            .default("30".into())
            .interact_text()?;
        match s.parse::<u32>() {
            Ok(n) if n > 0 => break n,
            _ => eprintln!("  {} Must be >= 1.", "!".red()),
        }
    };

    // -- Collateral ----------------------------------------------------------

    println!("\n  {}", "Collateral".bold());

    let currency_labels = &["xmr  (Monero DLEQ -- default)", "zec  (Zcash shielded Sapling DLEQ)", "btc  (Bitcoin Taproot PTLC)", "eth  (Ethereum smart contract escrow)"];
    let currency_keys   = &["xmr", "zec", "btc", "eth"];
    let currency_idx = Select::with_theme(&theme)
        .with_prompt("Collateral currency")
        .items(currency_labels)
        .default(0)
        .interact()?;
    let currency = currency_keys[currency_idx].to_owned();
    let lock_mechanism = lock_mechanism_for_currency(&currency);

    let chain_id: Option<u32> = if currency == "eth" {
        println!("\n  {}", "Ethereum network".bold());
        let chain_labels = &["mainnet (1)", "Sepolia testnet (11155111)", "Holesky testnet (17000)"];
        let chain_ids    = &[1u32, 11155111u32, 17000u32];
        let chain_idx = Select::with_theme(&theme)
            .with_prompt("Target chain")
            .items(chain_labels)
            .default(0)
            .interact()?;
        Some(chain_ids[chain_idx])
    } else {
        None
    };

    let amount: String = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt(format!("Collateral per file ({}, e.g. 1.0)", currency.to_uppercase()))
            .default("1.0".into())
            .interact_text()?;
        if Decimal::from_str(&s).is_ok() { break s; }
        eprintln!("  {} Enter a valid decimal (e.g. 1.0).", "!".red());
    };

    let redemption_raw: String = Input::with_theme(&theme)
        .with_prompt("Redemption value at issuance -- fiat peg (e.g. 1000 USD; Enter to skip)")
        .allow_empty(true)
        .interact_text()?;
    let redemption_value = if redemption_raw.is_empty() { None } else { Some(redemption_raw) };

    let unit_count: u64 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Number of .bounty files to issue (units in the series)")
            .default("1".into())
            .interact_text()?;
        match s.parse::<u64>() {
            Ok(n) if n > 0 => break n,
            _ => eprintln!("  {} Must be a positive integer.", "!".red()),
        }
    };

    // -- Judge panel --------------------------------------------------------

    println!("\n  {}", "Judge panel".bold());

    let quorum: u32 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Quorum (minimum signatures to settle)")
            .default("3".into())
            .interact_text()?;
        match s.parse::<u32>() {
            Ok(n) if n > 0 => break n,
            _ => eprintln!("  {} Must be >= 1.", "!".red()),
        }
    };

    let challenge_days: u32 = loop {
        let s: String = Input::with_theme(&theme)
            .with_prompt("Challenge period (days)")
            .default("7".into())
            .interact_text()?;
        match s.parse::<u32>() {
            Ok(n) => break n,
            _ => eprintln!("  {} Must be a non-negative integer.", "!".red()),
        }
    };

    let mut judges: Vec<OracleEntry> = Vec::new();
    loop {
        println!("\n  Judge {}:", judges.len() + 1);

        let pubkey: String = loop {
            let s: String = Input::with_theme(&theme)
                .with_prompt("  Public key (32-byte hex, 64 chars)")
                .interact_text()?;
            if validate_hex_key(&s).is_ok() { break s; }
            eprintln!("  {} Must be 32-byte hex (64 hex characters).", "!".red());
        };

        let role_labels = &["quantitative", "qualitative", "both"];
        let role_idx = Select::with_theme(&theme)
            .with_prompt("  Role")
            .items(role_labels)
            .default(2)
            .interact()?;
        let role = role_labels[role_idx].to_owned();

        let fee_xmr: String = loop {
            let s: String = Input::with_theme(&theme)
                .with_prompt("  Attestation fee (XMR per event)")
                .default("0.01".into())
                .interact_text()?;
            if Decimal::from_str(&s).is_ok() { break s; }
            eprintln!("  {} Enter a valid decimal.", "!".red());
        };

        judges.push(OracleEntry { pubkey, role, fee_xmr });

        if judges.len() >= quorum as usize {
            let add_more = Confirm::with_theme(&theme)
                .with_prompt("Add another judge?")
                .default(false)
                .interact()?;
            if !add_more { break; }
        }
    }

    // -- Output directory ----------------------------------------------------

    let output_dir_s: String = Input::with_theme(&theme)
        .with_prompt("Output directory for .bounty files")
        .default(".".into())
        .interact_text()?;
    let output_dir = PathBuf::from(output_dir_s);
    if !output_dir.is_dir() {
        return Err(anyhow!("Not a directory: {}", output_dir.display()));
    }

    Ok(WizardSpec {
        title,
        description,
        terms,
        term_aggregation,
        deadline,
        expiry,
        grace_days,
        currency:          currency.clone(),
        lock_mechanism,
        chain_id,
        amount,
        redemption_value,
        unit_count,
        alg_epoch:         0,
        key_algorithm:     key_algorithm_for_currency(&currency),
        enc_algorithm:     "x25519-chacha20poly1305".to_owned(),
        t_met:             None,
        t_fail:            None,
        lock_address:      None,
        oracle_eth_pubkey: None,
        holder_pubkey:     None,
        scalar_encrypted:  None,
        pq_holder_pubkey:  None,
        pq_scalar_enc:     None,
        judges,
        quorum,
        challenge_days,
        output_dir,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn print_term(t: &Term, label: &str) {
    println!("  {}[{}] {}", label, t.goal_type, t.criterion);
    if let Some(did) = &t.data_id {
        let op  = t.operator.as_deref().unwrap_or("?");
        let thr = t.threshold.as_deref().unwrap_or("?");
        let pad = " ".repeat(label.len() + 2);
        println!("  {}data: {}  {} {}", pad, did.cyan(), op, thr);
        if let Some(a) = &t.aggregation { println!("  {}agg:  {}", pad, a); }
    }
}

fn print_spec_preview(s: &WizardSpec) {
    println!("  Title:       {}", s.title);
    println!("  Description: {}", s.description);
    if s.terms.len() == 1 {
        print_term(&s.terms[0], "Criterion:   ");
    } else {
        let agg = s.term_aggregation.as_deref().unwrap_or("AND");
        println!("  Terms ({}):", agg);
        for (i, t) in s.terms.iter().enumerate() {
            print_term(t, &format!("  {}. ", i + 1));
        }
    }
    println!("  Deadline:    {}", s.deadline);
    println!("  Expiry:      {}", s.expiry);
    println!("  Grace:       {} day(s) after expiry", s.grace_days);
    let total = Decimal::from_str(&s.amount).unwrap_or_default()
        * Decimal::from(s.unit_count);
    let cur = s.currency.to_uppercase();
    println!("  Currency:    {}  mechanism: {}", cur, s.lock_mechanism);
    if let Some(cid) = s.chain_id {
        println!("  Chain ID:    {}", cid);
    }
    println!("  Collateral:  {} {}/file x {} file(s) = {} {} total",
        s.amount, cur, s.unit_count, total, cur);
    if let Some(rv) = &s.redemption_value {
        println!("  Redeem val:  {}", rv);
    }
    println!("  Quorum:      {}/{}", s.quorum, s.judges.len());
    println!("  Challenge:   {} day(s)", s.challenge_days);
    println!("  Judges:");
    for o in &s.judges {
        let prefix = &o.pubkey[..8.min(o.pubkey.len())];
        println!("    {}...  role: {}  fee: {} XMR", prefix.cyan(), o.role, o.fee_xmr);
    }
    println!("  Output dir:  {}", s.output_dir.display().to_string().dimmed());
}

fn require_file(file: &PathBuf) -> Result<()> {
    if !file.exists() {
        Err(anyhow!("File not found: {}", file.display()))
    } else {
        Ok(())
    }
}

fn validate_hex_key(hex_str: &str) -> Result<()> {
    let bytes = hex::decode(hex_str.trim())
        .map_err(|e| anyhow!("Invalid hex '{}': {}", &hex_str[..8.min(hex_str.len())], e))?;
    if bytes.len() != 32 {
        return Err(anyhow!("Key must be 32 bytes, got {}", bytes.len()));
    }
    Ok(())
}
