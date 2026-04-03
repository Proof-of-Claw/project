use crate::core::config::AgentConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// EIP-8004 Trustless Agents integration client.
/// Handles agent identity registration, reputation queries, and validation recording
/// via the on-chain EIP-8004 registries.
pub struct EIP8004Client {
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

    /// Query an agent's reputation from the EIP-8004 Reputation Registry
    pub async fn get_reputation(
        &self,
        _agent_id: &[u8; 32],
        _tag: &str,
    ) -> Result<ReputationSummary> {
        // TODO: Call reputationRegistry.getSummary() via ethers-rs
        Ok(ReputationSummary {
            count: 0,
            summary_value: 0,
            summary_value_decimals: 0,
        })
    }

    /// Query an agent's validation history from the EIP-8004 Validation Registry
    pub async fn get_validation_summary(
        &self,
        _agent_id: &[u8; 32],
    ) -> Result<ValidationSummary> {
        // TODO: Call validationRegistry.getSummary() via ethers-rs
        Ok(ValidationSummary {
            count: 0,
            average_response: 0,
        })
    }

    /// Submit reputation feedback after a proven interaction
    pub async fn submit_feedback(
        &self,
        _agent_id: &[u8; 32],
        _feedback: &ReputationFeedback,
    ) -> Result<String> {
        // TODO: Call eip8004Integration.submitReputation() via ethers-rs
        Ok(String::new())
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
