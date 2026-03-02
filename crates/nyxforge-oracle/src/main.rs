use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use nyxforge_oracle::{OracleConfig, OracleNode};
use nyxforge_oracle::verifier::MockDataSource;
use nyxforge_core::types::PublicKey;

#[derive(Parser, Debug)]
#[command(version, about = "NyxForge oracle node")]
struct Args {
    /// Path to oracle secret key file (32-byte hex)
    #[arg(long)]
    key_file: Option<std::path::PathBuf>,

    /// Node RPC endpoint to submit attestations
    #[arg(long, default_value = "http://127.0.0.1:8888/rpc")]
    node_rpc: String,

    /// Poll interval in seconds
    #[arg(long, default_value_t = 3600)]
    poll_interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("nyxforge_oracle=info".parse()?))
        .init();

    tracing::info!("NyxForge oracle node starting");

    // TODO: load secret key from file or generate ephemeral.
    let secret_key = [0u8; 32];
    let public_key = PublicKey([0u8; 32]); // TODO: derive from secret key

    let config = OracleConfig {
        public_key,
        bond_filter:       vec![],
        poll_lead_secs:    86_400,
        poll_interval_secs: args.poll_interval,
    };

    let sources: Vec<Box<dyn nyxforge_oracle::verifier::DataSource>> = vec![
        // Register data adapters here.
        // Example: Box::new(HttpJsonSource::new("us.hud.pit_count", "https://...", "/value")),
        Box::new(MockDataSource {
            data_id: "example.metric".into(),
            value:   rust_decimal::Decimal::from(0u32),
        }),
    ];

    let _node = OracleNode::new(config, secret_key, sources);

    tracing::info!(%args.node_rpc, "oracle running — polling every {}s", args.poll_interval);

    // TODO: main loop — fetch registered bonds from node, evaluate, post attestations.
    futures::future::pending::<()>().await;
    Ok(())
}
