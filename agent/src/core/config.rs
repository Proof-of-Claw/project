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
    pub eip8004_identity_registry: Option<String>,
    pub eip8004_reputation_registry: Option<String>,
    pub eip8004_validation_registry: Option<String>,
    pub eip8004_integration_contract: Option<String>,
    pub inft_contract: Option<String>,
    pub risc_zero_image_id: Option<String>,
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
        let private_key = env::var("PRIVATE_KEY")
            .context("PRIVATE_KEY not set")?;

        // Reject the all-zeros placeholder key
        let stripped = private_key.trim_start_matches("0x");
        if stripped.chars().all(|c| c == '0') {
            anyhow::bail!(
                "PRIVATE_KEY is set to all zeros — configure a real private key in .env"
            );
        }

        Ok(Self {
            agent_id: env::var("AGENT_ID")
                .context("AGENT_ID not set")?,
            ens_name: env::var("ENS_NAME")
                .context("ENS_NAME not set")?,
            private_key,
            rpc_url: env::var("RPC_URL")
                .context("RPC_URL not set — provide a real RPC endpoint (e.g. Alchemy, Infura)")?,
            zero_g_indexer_rpc: env::var("ZERO_G_INDEXER_RPC")
                .context("ZERO_G_INDEXER_RPC not set")?,
            zero_g_compute_endpoint: env::var("ZERO_G_COMPUTE_ENDPOINT")
                .context("ZERO_G_COMPUTE_ENDPOINT not set")?,
            dm3_delivery_service_url: env::var("DM3_DELIVERY_SERVICE_URL")
                .context("DM3_DELIVERY_SERVICE_URL not set")?,
            ledger_origin_token: env::var("LEDGER_ORIGIN_TOKEN").ok(),
            eip8004_identity_registry: non_zero_address("EIP8004_IDENTITY_REGISTRY"),
            eip8004_reputation_registry: non_zero_address("EIP8004_REPUTATION_REGISTRY"),
            eip8004_validation_registry: non_zero_address("EIP8004_VALIDATION_REGISTRY"),
            eip8004_integration_contract: non_zero_address("EIP8004_INTEGRATION_CONTRACT"),
            inft_contract: non_zero_address("INFT_CONTRACT"),
            risc_zero_image_id: non_zero_hash("RISC_ZERO_IMAGE_ID"),
            policy: PolicyConfig {
                allowed_tools: env::var("ALLOWED_TOOLS")
                    .context("ALLOWED_TOOLS not set")?
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                endpoint_allowlist: env::var("ENDPOINT_ALLOWLIST")
                    .context("ENDPOINT_ALLOWLIST not set")?
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                max_value_autonomous_wei: env::var("MAX_VALUE_AUTONOMOUS_WEI")
                    .context("MAX_VALUE_AUTONOMOUS_WEI not set")?
                    .parse()
                    .context("MAX_VALUE_AUTONOMOUS_WEI must be a valid u64")?,
            },
        })
    }
}

/// Read an env var as an address, returning None if unset or all-zeros.
fn non_zero_address(key: &str) -> Option<String> {
    env::var(key).ok().and_then(|v| {
        let stripped = v.trim_start_matches("0x");
        if stripped.is_empty() || stripped.chars().all(|c| c == '0') {
            None
        } else {
            Some(v)
        }
    })
}

/// Read an env var as a hash, returning None if unset or all-zeros.
fn non_zero_hash(key: &str) -> Option<String> {
    env::var(key).ok().and_then(|v| {
        let stripped = v.trim_start_matches("0x");
        if stripped.is_empty() || stripped.chars().all(|c| c == '0') {
            None
        } else {
            Some(v)
        }
    })
}
