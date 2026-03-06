//! P2Pool mini stratum client — TCP, line-delimited JSON (Stratum v1 compatible).
//!
//! Protocol flow:
//!   client → login {login, pass, agent}
//!   server ← result {id, job: {blob, job_id, target, seed_hash, height}}
//!   server ← job    {blob, job_id, target, seed_hash, height}   (push)
//!   client → submit {id, job_id, nonce, result}

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

/// Default P2Pool stratum endpoint (local P2Pool instance).
/// For stagenet: run `p2pool --stagenet --host 127.0.0.1` and point here.
pub const DEFAULT_P2POOL_URL: &str = "127.0.0.1:3333";

/// Miner configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerConfig {
    /// Number of CPU threads to use.
    pub threads: usize,
    /// P2Pool stratum endpoint, e.g. `"p2pool.io:3333"`.
    pub p2pool_url: String,
    /// XMR payout address (primary address, no payment ID).
    pub xmr_address: String,
    /// DRK payout address (for future DarkFi merge-mine).
    pub drk_address: String,
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self {
            threads: 1,
            p2pool_url: DEFAULT_P2POOL_URL.to_string(),
            xmr_address: String::new(),
            drk_address: String::new(),
        }
    }
}

/// A mining job received from P2Pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_id: String,
    pub blob: String,       // hex-encoded block template
    pub target: String,     // compact difficulty target
    pub seed_hash: String,  // RandomX seed hash (hex)
    pub height: u64,
}

/// A submitted share.
#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    pub worker_id: String,
    pub job_id: String,
    pub nonce: String,  // hex, 4 bytes, little-endian
    pub result: String, // hex hash result
}

// ---------------------------------------------------------------------------
// Internal protocol structs (Stratum v1 JSON)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct LoginRequest<'a> {
    id: u32,
    method: &'a str,
    params: LoginParams<'a>,
}

#[derive(Debug, Serialize)]
struct LoginParams<'a> {
    login: &'a str,
    pass: &'a str,
    agent: &'a str,
}

#[derive(Debug, Serialize)]
struct SubmitRequest<'a> {
    id: u32,
    method: &'a str,
    params: &'a Share,
}

#[derive(Debug, Deserialize)]
struct ServerMessage {
    #[serde(default)]
    id: Option<u32>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    result: Option<LoginResult>,
    #[serde(default)]
    params: Option<Job>,
    #[serde(default)]
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct LoginResult {
    id: String,
    #[serde(default)]
    job: Option<Job>,
}

// ---------------------------------------------------------------------------
// Stratum client
// ---------------------------------------------------------------------------

/// Active connection to a P2Pool stratum server.
pub struct StratumClient {
    worker_id: String,
    reader: BufReader<tokio::net::tcp::OwnedReadHalf>,
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl StratumClient {
    /// Connect and log in to the stratum server.
    pub async fn connect(config: &MinerConfig) -> Result<(Self, Job)> {
        info!(url = %config.p2pool_url, "connecting to P2Pool stratum");

        let stream = TcpStream::connect(&config.p2pool_url).await
            .map_err(|e| anyhow!("cannot connect to {}: {e}", config.p2pool_url))?;
        let (rd, mut wr) = stream.into_split();
        let mut reader = BufReader::new(rd);

        // Send login
        let login = LoginRequest {
            id: 1,
            method: "login",
            params: LoginParams {
                login: &config.xmr_address,
                pass: "x",
                agent: "nyxforge-miner/0.1.0",
            },
        };
        let login_json = serde_json::to_string(&login)? + "\n";
        wr.write_all(login_json.as_bytes()).await?;
        debug!("sent login");

        // Read login response
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        debug!(response = %line.trim(), "login response");

        let msg: ServerMessage = serde_json::from_str(line.trim())
            .map_err(|e| anyhow!("invalid login response: {e}\nraw: {line}"))?;

        if let Some(err) = msg.error {
            return Err(anyhow!("stratum login error: {err}"));
        }

        let result = msg.result.ok_or_else(|| anyhow!("no result in login response"))?;
        let worker_id = result.id.clone();
        let first_job = result.job.ok_or_else(|| anyhow!("no job in login response"))?;

        info!(worker_id = %worker_id, height = first_job.height, "logged in to P2Pool");

        Ok((Self { worker_id, reader, writer: wr }, first_job))
    }

    /// Read the next job pushed by the server (blocks until one arrives).
    pub async fn next_job(&mut self) -> Result<Job> {
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line).await?;
            if n == 0 {
                return Err(anyhow!("stratum connection closed"));
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            debug!(msg = %line, "stratum message");

            let msg: ServerMessage = serde_json::from_str(line)
                .map_err(|e| { warn!("failed to parse: {e} — {line}"); e })?;

            // Push job
            if msg.method.as_deref() == Some("job") {
                if let Some(job) = msg.params {
                    return Ok(job);
                }
            }
        }
    }

    /// Submit a found share.
    pub async fn submit_share(&mut self, share: &Share) -> Result<()> {
        let req = SubmitRequest { id: 2, method: "submit", params: share };
        let json = serde_json::to_string(&req)? + "\n";
        self.writer.write_all(json.as_bytes()).await?;
        debug!(job_id = %share.job_id, nonce = %share.nonce, "share submitted");
        Ok(())
    }

    pub fn worker_id(&self) -> &str {
        &self.worker_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- MinerConfig defaults ---

    #[test]
    fn default_p2pool_url_is_local() {
        let cfg = MinerConfig::default();
        assert_eq!(cfg.p2pool_url, DEFAULT_P2POOL_URL);
        assert!(cfg.p2pool_url.starts_with("127.0.0.1"));
    }

    #[test]
    fn default_thread_count_is_one() {
        assert_eq!(MinerConfig::default().threads, 1);
    }

    #[test]
    fn miner_config_roundtrips_via_json() {
        let cfg = MinerConfig {
            threads: 4,
            p2pool_url: "192.168.1.1:3333".into(),
            xmr_address: "5Abcdef".into(),
            drk_address: "deadbeef".into(),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let restored: MinerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.threads, 4);
        assert_eq!(restored.p2pool_url, "192.168.1.1:3333");
        assert_eq!(restored.xmr_address, "5Abcdef");
        assert_eq!(restored.drk_address, "deadbeef");
    }

    // --- Share serialization ---

    #[test]
    fn share_serialises_expected_fields() {
        let share = Share {
            worker_id: "w1".into(),
            job_id:    "j42".into(),
            nonce:     "0a0b0c0d".into(),
            result:    "abcd1234".into(),
        };
        let json = serde_json::to_string(&share).unwrap();
        assert!(json.contains("\"worker_id\""));
        assert!(json.contains("\"job_id\""));
        assert!(json.contains("\"nonce\""));
        assert!(json.contains("\"result\""));
        assert!(json.contains("j42"));
        assert!(json.contains("0a0b0c0d"));
    }

    // --- Job deserialization ---

    #[test]
    fn job_deserialises_from_json() {
        let raw = r#"{
            "job_id":    "abc123",
            "blob":      "deadbeef",
            "target":    "0000ffff",
            "seed_hash": "cafebabe00000000000000000000000000000000000000000000000000000000",
            "height":    12345
        }"#;
        let job: Job = serde_json::from_str(raw).unwrap();
        assert_eq!(job.job_id, "abc123");
        assert_eq!(job.blob, "deadbeef");
        assert_eq!(job.target, "0000ffff");
        assert_eq!(job.height, 12345);
    }

    #[test]
    fn job_roundtrips_via_json() {
        let job = Job {
            job_id:    "xyz".into(),
            blob:      "aabbcc".into(),
            target:    "00001000".into(),
            seed_hash: "00".repeat(32),
            height:    999,
        };
        let json = serde_json::to_string(&job).unwrap();
        let restored: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.job_id, "xyz");
        assert_eq!(restored.height, 999);
    }
}
