use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub ens_name: String,
    pub private_key: String,
    pub rpc_url: String,
    pub zero_g_indexer_rpc: String,
    pub zero_g_compute_endpoint: String,
    pub dm3_delivery_service_url: String,
    pub ledger_origin_token: Option<String>,
    pub policy: PolicyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub allowed_tools: Vec<String>,
    pub endpoint_allowlist: Vec<String>,
    pub max_value_autonomous_wei: u64,
}

impl AgentConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            agent_id: env::var("AGENT_ID")
                .context("AGENT_ID not set")?,
            ens_name: env::var("ENS_NAME")
                .context("ENS_NAME not set")?,
            private_key: env::var("PRIVATE_KEY")
                .context("PRIVATE_KEY not set")?,
            rpc_url: env::var("RPC_URL")
                .unwrap_or_else(|_| "https://eth-sepolia.g.alchemy.com/v2/demo".to_string()),
            zero_g_indexer_rpc: env::var("ZERO_G_INDEXER_RPC")
                .unwrap_or_else(|_| "https://indexer-storage-testnet.0g.ai".to_string()),
            zero_g_compute_endpoint: env::var("ZERO_G_COMPUTE_ENDPOINT")
                .unwrap_or_else(|_| "https://broker-testnet.0g.ai".to_string()),
            dm3_delivery_service_url: env::var("DM3_DELIVERY_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            ledger_origin_token: env::var("LEDGER_ORIGIN_TOKEN").ok(),
            policy: PolicyConfig {
                allowed_tools: env::var("ALLOWED_TOOLS")
                    .unwrap_or_else(|_| "swap,transfer,query".to_string())
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                endpoint_allowlist: env::var("ENDPOINT_ALLOWLIST")
                    .unwrap_or_else(|_| "https://api.uniswap.org,https://api.0x.org".to_string())
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                max_value_autonomous_wei: env::var("MAX_VALUE_AUTONOMOUS_WEI")
                    .unwrap_or_else(|_| "1000000000000000000".to_string())
                    .parse()
                    .unwrap_or(1_000_000_000_000_000_000),
            },
        })
    }
}
