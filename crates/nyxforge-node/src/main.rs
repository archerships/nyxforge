use anyhow::Result;
use clap::Parser;
use tracing_subscriber::EnvFilter;

mod miner;
mod p2p;
mod rpc;
mod state;
mod sync;
mod wallet_manager;

/// NyxForge P2P node — anonymous social policy bond market
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Connect to the DarkFi testnet (default: mainnet when live)
    #[arg(long)]
    testnet: bool,

    /// Listen address for the P2P swarm
    #[arg(long, default_value = "/ip4/0.0.0.0/tcp/7777")]
    listen: String,

    /// Bootstrap peers (multiaddr format), comma-separated
    #[arg(long, value_delimiter = ',')]
    bootstrap: Vec<String>,

    /// JSON-RPC listen address for local clients (browser WASM)
    #[arg(long, default_value = "127.0.0.1:8888")]
    rpc_addr: String,

    /// Path to node data directory
    #[arg(long, default_value = "node_data")]
    data_dir: std::path::PathBuf,

    /// Remote monerod URL for the XMR light wallet
    #[arg(long, default_value = "http://node.community.rino.io:18081")]
    xmr_node: String,

    /// P2Pool mini stratum endpoint
    #[arg(long, default_value = "p2pool.io:3333")]
    p2pool_url: String,

    /// CPU threads to use for mining
    #[arg(long, default_value_t = 1)]
    mine_threads: usize,

    /// Begin mining immediately at launch
    #[arg(long)]
    mine_on_start: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("nyxforge=info".parse()?)
                .add_directive("libp2p=warn".parse()?),
        )
        .init();

    tracing::info!("NyxForge node starting");
    tracing::info!(testnet = args.testnet, listen = %args.listen, "configuration");

    // Initialise shared state.
    let state = state::NodeState::new(&args.data_dir).await?;

    // Start P2P swarm.
    let p2p_handle = {
        let state = state.clone();
        let listen = args.listen.clone();
        let bootstrap = args.bootstrap.clone();
        tokio::spawn(async move { p2p::run_swarm(state, &listen, &bootstrap).await })
    };

    // Start RPC server.
    let rpc_handle = {
        let state = state.clone();
        let addr = args.rpc_addr.clone();
        tokio::spawn(async move { rpc::run_server(state, &addr).await })
    };

    // Start wallet background scanner.
    let wallet_handle = {
        let state = state.clone();
        let xmr_node = args.xmr_node.clone();
        tokio::spawn(async move { wallet_manager::run(state, xmr_node).await })
    };

    // Start miner control task.
    let miner_handle = {
        let state = state.clone();
        let miner_config = nyxforge_miner::MinerConfig {
            threads: args.mine_threads,
            p2pool_url: args.p2pool_url.clone(),
            xmr_address: String::new(), // populated after wallet.create
            drk_address: String::new(),
        };
        let on_start = args.mine_on_start;
        tokio::spawn(async move { miner::run(state, miner_config, on_start).await })
    };

    tokio::select! {
        r = p2p_handle    => { tracing::error!("P2P task exited: {:?}", r); }
        r = rpc_handle    => { tracing::error!("RPC task exited: {:?}", r); }
        r = wallet_handle => { tracing::error!("Wallet task exited: {:?}", r); }
        r = miner_handle  => { tracing::error!("Miner task exited: {:?}", r); }
    }

    Ok(())
}
