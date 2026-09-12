use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::path::PathBuf;

use crate::rpc_client::RpcClient;

// ---------------------------------------------------------------------------
// Clap types
// ---------------------------------------------------------------------------

#[derive(Parser)]
pub struct JudgeArgs {
    #[command(subcommand)]
    pub command: JudgeCommand,
}

#[derive(Subcommand)]
pub enum JudgeCommand {
    /// Review GoalSpec and accept judge responsibility
    Accept {
        /// Path to the .bounty file
        file: PathBuf,
    },
    /// Decline judge responsibility with a reason
    Reject {
        /// Path to the .bounty file
        file: PathBuf,
    },
    /// Submit evidence and sign an attestation
    Attest {
        /// Path to the .bounty file
        file: PathBuf,
    },
    /// Show each judge's accept/reject/attestation status
    Status {
        /// Path to the .bounty file
        file: PathBuf,
    },
    /// List .bounty files this judge key is registered on
    List {
        /// Directory to scan (default: current directory)
        #[arg(default_value = ".")]
        dir: PathBuf,
        /// Judge public key hex; uses key at --judge-key if omitted
        #[arg(long)]
        key: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub async fn run(args: JudgeArgs, _rpc: &RpcClient) -> Result<()> {
    match args.command {
        JudgeCommand::Accept { file }       => cmd_accept(&file).await,
        JudgeCommand::Reject { file }       => cmd_reject(&file).await,
        JudgeCommand::Attest { file }       => cmd_attest(&file).await,
        JudgeCommand::Status { file }       => cmd_status(&file).await,
        JudgeCommand::List { dir, key }     => cmd_list(&dir, key.as_deref()).await,
    }
}

// ---------------------------------------------------------------------------
// judge accept
// ---------------------------------------------------------------------------

async fn cmd_accept(file: &PathBuf) -> Result<()> {
    require_file(file)?;
    // TODO(phase-3): read judges table, verify caller key is registered, write
    //   accepted_at timestamp to judges row.  If all judges have accepted and
    //   bounty is DRAFT, transition state to ACTIVE.
    println!("  {} judge accept requires the nyxforge-bounty crate (Phase 3).", "!".yellow());
    println!("  File: {}", file.display().to_string().cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// judge reject
// ---------------------------------------------------------------------------

async fn cmd_reject(file: &PathBuf) -> Result<()> {
    require_file(file)?;
    let theme = ColorfulTheme::default();
    let reason: String = Input::with_theme(&theme)
        .with_prompt("Reason for rejection (required)")
        .interact_text()?;
    if reason.trim().is_empty() {
        return Err(anyhow!("A reason is required when rejecting a bounty."));
    }
    // TODO(phase-3): write rejection and reason to bounty file; issuer must revise
    //   judge panel before resubmitting.
    println!("  {} judge reject requires the nyxforge-bounty crate (Phase 3).", "!".yellow());
    println!("  File:   {}", file.display().to_string().cyan());
    println!("  Reason: {}", reason.dimmed());
    Ok(())
}

// ---------------------------------------------------------------------------
// judge attest
// ---------------------------------------------------------------------------

async fn cmd_attest(file: &PathBuf) -> Result<()> {
    require_file(file)?;
    let theme = ColorfulTheme::default();

    let result_labels = &["met", "not_met"];
    let result_idx = Select::with_theme(&theme)
        .with_prompt("Attestation result")
        .items(result_labels)
        .default(0)
        .interact()?;
    let result = result_labels[result_idx];

    let evidence_path_s: String = Input::with_theme(&theme)
        .with_prompt("Evidence file path (PDF, JSON, video; Enter to skip)")
        .allow_empty(true)
        .interact_text()?;

    // TODO(phase-3): if evidence_path given, read file, write BLOB + sha256 to
    //   evidence table.  Sign (bounty_id || result || evidence_sha256) with judge
    //   key.  Write row to attestations table.  If quorum reached, transition bounty
    //   state to REDEEMABLE in history table.
    println!("  {} judge attest requires the nyxforge-bounty crate (Phase 3).", "!".yellow());
    println!("  File:   {}", file.display().to_string().cyan());
    println!("  Result: {}", result);
    if !evidence_path_s.is_empty() {
        println!("  Evidence: {}", evidence_path_s.dimmed());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// judge status
// ---------------------------------------------------------------------------

async fn cmd_status(file: &PathBuf) -> Result<()> {
    require_file(file)?;
    // TODO(phase-3): read judges table (pubkey, role, accepted_at) and
    //   attestations table (oracle_pubkey, result, signed_at).  Print one row
    //   per judge: key prefix, role, accept status, attestation result if any.
    println!("  {} judge status requires the nyxforge-bounty crate (Phase 3).", "!".yellow());
    println!("  File: {}", file.display().to_string().cyan());
    Ok(())
}

// ---------------------------------------------------------------------------
// judge list
// ---------------------------------------------------------------------------

async fn cmd_list(dir: &PathBuf, key: Option<&str>) -> Result<()> {
    if !dir.is_dir() {
        return Err(anyhow!("Not a directory: {}", dir.display()));
    }
    if let Some(k) = key {
        println!("  Key: {}...", &k[..8.min(k.len())].cyan());
    }
    // TODO(phase-3): for each .bounty file in dir, read judges table and check
    //   whether this key appears.  Print matching files with accept/attest status.
    println!("  {} judge list requires the nyxforge-bounty crate (Phase 3).", "!".yellow());
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn require_file(file: &PathBuf) -> Result<()> {
    if !file.exists() {
        Err(anyhow!("File not found: {}", file.display()))
    } else {
        Ok(())
    }
}
