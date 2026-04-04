use crate::core::config::AgentConfig;
use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// EIP-8004 Trustless Agents integration client.
/// Handles agent identity registration, reputation queries, and validation recording
/// via the on-chain EIP-8004 registries.
pub struct EIP8004Client {
    client: Client,
    identity_registry: String,
    reputation_registry: String,
    validation_registry: String,
    integration_contract: String,
    rpc_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    #[serde(rename = "type")]
    pub registration_type: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub services: Vec<ServiceEndpoint>,
    pub x402_support: bool,
    pub active: bool,
    pub registrations: Vec<RegistrationEntry>,
    pub supported_trust: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub name: String,
    pub endpoint: String,
    pub version: Option<String>,
    pub skills: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationEntry {
    pub agent_id: u64,
    pub agent_registry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationSummary {
    pub count: u64,
    pub summary_value: i128,
    pub summary_value_decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub count: u64,
    pub average_response: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationFeedback {
    pub value: i128,
    pub value_decimals: u8,
    pub tag1: String,
    pub tag2: String,
    pub endpoint: String,
    pub feedback_uri: String,
}

impl EIP8004Client {
    pub async fn new(config: &AgentConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            identity_registry: config
                .eip8004_identity_registry
                .clone()
                .unwrap_or_default(),
            reputation_registry: config
                .eip8004_reputation_registry
                .clone()
                .unwrap_or_default(),
            validation_registry: config
                .eip8004_validation_registry
                .clone()
                .unwrap_or_default(),
            integration_contract: config
                .eip8004_integration_contract
                .clone()
                .unwrap_or_default(),
            rpc_url: config.rpc_url.clone(),
        })
    }

    /// Make an eth_call to a contract and return the raw hex result
    async fn eth_call(&self, to: &str, calldata: &[u8]) -> Result<String> {
        let resp = self
            .client
            .post(&self.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_call",
                "params": [{
                    "to": to,
                    "data": format!("0x{}", hex::encode(calldata))
                }, "latest"],
                "id": 1
            }))
            .send()
            .await
            .context("Failed to send eth_call to RPC")?;

        let body: serde_json::Value = resp.json().await?;

        if let Some(error) = body.get("error") {
            anyhow::bail!("RPC error: {}", error);
        }

        let result = body["result"]
            .as_str()
            .unwrap_or("0x")
            .to_string();

        Ok(result)
    }

    /// Send a signed transaction and return the tx hash
    async fn send_transaction(&self, to: &str, calldata: &[u8]) -> Result<String> {
        // Build raw transaction via eth_sendRawTransaction
        // For now we use eth_sendTransaction which requires an unlocked account on the node,
        // or the caller pre-signs. In production, use ethers-rs Wallet + SignerMiddleware.
        let resp = self
            .client
            .post(&self.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_sendTransaction",
                "params": [{
                    "to": to,
                    "data": format!("0x{}", hex::encode(calldata))
                }],
                "id": 1
            }))
            .send()
            .await
            .context("Failed to send transaction to RPC")?;

        let body: serde_json::Value = resp.json().await?;

        if let Some(error) = body.get("error") {
            anyhow::bail!("RPC error: {}", error);
        }

        let tx_hash = body["result"]
            .as_str()
            .context("No tx hash in RPC response")?
            .to_string();

        Ok(tx_hash)
    }

    /// Build the agent's EIP-8004 registration file
    pub fn build_registration(
        &self,
        config: &AgentConfig,
        token_id: u64,
        chain_id: u64,
    ) -> AgentRegistration {
        AgentRegistration {
            registration_type: "https://eips.ethereum.org/EIPS/eip-8004#registration-v1"
                .to_string(),
            name: config.ens_name.clone(),
            description: format!(
                "Proof of Claw agent with policy-compliant execution proven via RISC Zero."
            ),
            image: String::new(),
            services: vec![
                ServiceEndpoint {
                    name: "ENS".to_string(),
                    endpoint: config.ens_name.clone(),
                    version: None,
                    skills: None,
                },
                ServiceEndpoint {
                    name: "DM3".to_string(),
                    endpoint: config.dm3_delivery_service_url.clone(),
                    version: Some("1.0".to_string()),
                    skills: None,
                },
            ],
            x402_support: false,
            active: true,
            registrations: vec![RegistrationEntry {
                agent_id: token_id,
                agent_registry: format!(
                    "eip155:{}:{}",
                    chain_id, self.identity_registry
                ),
            }],
            supported_trust: vec![
                "reputation".to_string(),
                "validation-zk".to_string(),
            ],
        }
    }

    /// Query an agent's reputation from the EIP-8004 Reputation Registry.
    ///
    /// Calls `getSummary(bytes32 agentId, string tag)` on the reputation registry contract.
    pub async fn get_reputation(
        &self,
        agent_id: &[u8; 32],
        tag: &str,
    ) -> Result<ReputationSummary> {
        if self.reputation_registry.is_empty() {
            anyhow::bail!("EIP-8004 reputation registry address not configured");
        }

        // getSummary(bytes32,string) selector
        let selector = &ethers::utils::keccak256(b"getSummary(bytes32,string)")[..4];
        let encoded = ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(agent_id.to_vec()),
            ethers::abi::Token::String(tag.to_string()),
        ]);

        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&encoded);

        let result_hex = self.eth_call(&self.reputation_registry, &calldata).await?;
        let result_bytes = hex::decode(result_hex.trim_start_matches("0x")).unwrap_or_default();

        if result_bytes.len() < 96 {
            // No data returned — agent has no reputation records
            return Ok(ReputationSummary {
                count: 0,
                summary_value: 0,
                summary_value_decimals: 0,
            });
        }

        // Decode (uint256 count, int256 summaryValue, uint8 summaryValueDecimals)
        let tokens = ethers::abi::decode(
            &[
                ethers::abi::ParamType::Uint(256),
                ethers::abi::ParamType::Int(256),
                ethers::abi::ParamType::Uint(8),
            ],
            &result_bytes,
        )
        .context("Failed to decode reputation summary")?;

        let count = tokens[0].clone().into_uint().unwrap_or_default().as_u64();
        let summary_value = {
            let raw = tokens[1].clone().into_int().unwrap_or_default();
            // Convert U256 to i128 (signed)
            let bytes = raw.as_u128();
            if raw.bit(255) {
                -((!bytes).wrapping_add(1) as i128)
            } else {
                bytes as i128
            }
        };
        let summary_value_decimals = tokens[2].clone().into_uint().unwrap_or_default().as_u64() as u8;

        Ok(ReputationSummary {
            count,
            summary_value,
            summary_value_decimals,
        })
    }

    /// Query an agent's validation history from the EIP-8004 Validation Registry.
    ///
    /// Calls `getSummary(bytes32 agentId)` on the validation registry contract.
    pub async fn get_validation_summary(
        &self,
        agent_id: &[u8; 32],
    ) -> Result<ValidationSummary> {
        if self.validation_registry.is_empty() {
            anyhow::bail!("EIP-8004 validation registry address not configured");
        }

        // getSummary(bytes32) selector
        let selector = &ethers::utils::keccak256(b"getSummary(bytes32)")[..4];
        let encoded = ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(agent_id.to_vec()),
        ]);

        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&encoded);

        let result_hex = self.eth_call(&self.validation_registry, &calldata).await?;
        let result_bytes = hex::decode(result_hex.trim_start_matches("0x")).unwrap_or_default();

        if result_bytes.len() < 64 {
            return Ok(ValidationSummary {
                count: 0,
                average_response: 0,
            });
        }

        // Decode (uint256 count, uint8 averageResponse)
        let tokens = ethers::abi::decode(
            &[
                ethers::abi::ParamType::Uint(256),
                ethers::abi::ParamType::Uint(8),
            ],
            &result_bytes,
        )
        .context("Failed to decode validation summary")?;

        let count = tokens[0].clone().into_uint().unwrap_or_default().as_u64();
        let average_response = tokens[1].clone().into_uint().unwrap_or_default().as_u64() as u8;

        Ok(ValidationSummary {
            count,
            average_response,
        })
    }

    /// Submit reputation feedback after a proven interaction.
    ///
    /// Calls `submitReputation(bytes32 agentId, int256 value, uint8 valueDecimals, string tag1, string tag2, string endpoint, string feedbackUri)`
    /// on the EIP-8004 integration contract.
    pub async fn submit_feedback(
        &self,
        agent_id: &[u8; 32],
        feedback: &ReputationFeedback,
    ) -> Result<String> {
        if self.integration_contract.is_empty() {
            anyhow::bail!("EIP-8004 integration contract address not configured");
        }

        let selector = &ethers::utils::keccak256(
            b"submitReputation(bytes32,int256,uint8,string,string,string,string)",
        )[..4];

        // Convert i128 value to ethers I256
        let value_token = if feedback.value >= 0 {
            ethers::abi::Token::Int(ethers::types::U256::from(feedback.value as u128))
        } else {
            // Two's complement for negative values
            let abs = (-feedback.value) as u128;
            let twos_complement = ethers::types::U256::MAX - ethers::types::U256::from(abs) + 1;
            ethers::abi::Token::Int(twos_complement)
        };

        let encoded = ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(agent_id.to_vec()),
            value_token,
            ethers::abi::Token::Uint(ethers::types::U256::from(feedback.value_decimals)),
            ethers::abi::Token::String(feedback.tag1.clone()),
            ethers::abi::Token::String(feedback.tag2.clone()),
            ethers::abi::Token::String(feedback.endpoint.clone()),
            ethers::abi::Token::String(feedback.feedback_uri.clone()),
        ]);

        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&encoded);

        let tx_hash = self
            .send_transaction(&self.integration_contract, &calldata)
            .await?;

        Ok(tx_hash)
    }

    /// Check if an agent meets minimum trust thresholds
    pub async fn meets_trust_threshold(
        &self,
        agent_id: &[u8; 32],
        min_reputation: i128,
        min_validation_score: u8,
    ) -> Result<bool> {
        let reputation = self.get_reputation(agent_id, "policyCompliance").await?;
        let validation = self.get_validation_summary(agent_id).await?;

        // New agents with no history pass by default (bootstrap phase)
        if reputation.count == 0 && validation.count == 0 {
            return Ok(true);
        }

        let rep_ok = reputation.count == 0 || reputation.summary_value >= min_reputation;
        let val_ok = validation.count == 0 || validation.average_response >= min_validation_score;

        Ok(rep_ok && val_ok)
    }
}
