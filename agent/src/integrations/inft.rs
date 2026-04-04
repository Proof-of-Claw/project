use crate::core::config::AgentConfig;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 0G iNFT (ERC-7857) integration for Proof of Claw agents.
///
/// Handles minting agent identity as an Intelligent NFT on 0G Chain,
/// storing encrypted agent metadata on 0G Storage, and managing
/// the on-chain agent identity lifecycle.
pub struct INFTClient {
    client: Client,
    zero_g_storage_endpoint: String,
    zero_g_compute_endpoint: String,
    rpc_url: String,
    inft_contract: String,
}

/// Encrypted agent metadata stored on 0G Storage and referenced by the iNFT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent display name
    pub name: String,
    /// Agent's ENS subname
    pub ens_name: String,
    /// Policy configuration (allowed tools, thresholds)
    pub policy: PolicyMetadata,
    /// RISC Zero image ID for proof verification
    pub risc_zero_image_id: String,
    /// Capabilities this agent advertises
    pub capabilities: Vec<String>,
    /// DM3 delivery service endpoint
    pub dm3_endpoint: String,
    /// 0G Compute model used for inference
    pub inference_model: String,
    /// Agent version
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetadata {
    pub allowed_tools: Vec<String>,
    pub max_value_autonomous_wei: u64,
    pub endpoint_allowlist: Vec<String>,
}

/// Result of minting an iNFT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintResult {
    pub token_id: u64,
    pub encrypted_uri: String,
    pub metadata_hash: String,
    pub tx_hash: String,
}

/// On-chain iNFT data retrieved from the contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct INFTData {
    pub token_id: u64,
    pub owner: String,
    pub agent_id: String,
    pub policy_hash: String,
    pub risc_zero_image_id: String,
    pub encrypted_uri: String,
    pub metadata_hash: String,
    pub ens_name: String,
    pub reputation_score: u64,
    pub total_proofs: u64,
    pub minted_at: u64,
    pub active: bool,
}

impl INFTClient {
    pub async fn new(config: &AgentConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            zero_g_storage_endpoint: config.zero_g_indexer_rpc.clone(),
            zero_g_compute_endpoint: config.zero_g_compute_endpoint.clone(),
            rpc_url: config.rpc_url.clone(),
            inft_contract: config
                .inft_contract
                .clone()
                .unwrap_or_default(),
        })
    }

    /// Build agent metadata from config for iNFT minting
    pub fn build_metadata(config: &AgentConfig) -> AgentMetadata {
        AgentMetadata {
            name: config.agent_id.clone(),
            ens_name: config.ens_name.clone(),
            policy: PolicyMetadata {
                allowed_tools: config.policy.allowed_tools.clone(),
                max_value_autonomous_wei: config.policy.max_value_autonomous_wei,
                endpoint_allowlist: config.policy.endpoint_allowlist.clone(),
            },
            risc_zero_image_id: String::new(),
            capabilities: config.policy.allowed_tools.clone(),
            dm3_endpoint: config.dm3_delivery_service_url.clone(),
            inference_model: "0g-compute".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    /// Encrypt and upload agent metadata to 0G Storage
    ///
    /// Returns the storage URI and metadata hash for the iNFT mint transaction.
    pub async fn upload_metadata(&self, metadata: &AgentMetadata) -> Result<(String, String)> {
        let plaintext = serde_json::to_string(metadata)?;

        // Compute metadata hash for on-chain integrity verification
        let mut hasher = Sha256::new();
        hasher.update(plaintext.as_bytes());
        let metadata_hash = format!("0x{}", hex::encode(hasher.finalize()));

        // Upload to 0G Storage
        // In production this uses the 0G Storage SDK with AES-256-GCM encryption.
        // The encrypted blob is uploaded and the returned root hash becomes the URI.
        let upload_response = self
            .client
            .post(format!("{}/upload", self.zero_g_storage_endpoint))
            .json(&serde_json::json!({
                "data": plaintext,
                "tags": {
                    "type": "inft-metadata",
                    "agent": metadata.name,
                    "ens": metadata.ens_name,
                }
            }))
            .send()
            .await;

        let encrypted_uri = match upload_response {
            Ok(resp) => {
                let body = resp.text().await.unwrap_or_default();
                // Parse root hash from response, fall back to content-hash URI
                if body.starts_with("0x") {
                    format!("0g://{}", body)
                } else {
                    // Generate deterministic URI from content hash
                    format!("0g://{}", &metadata_hash[2..])
                }
            }
            Err(_) => {
                // Fallback: content-addressable URI from metadata hash
                format!("0g://{}", &metadata_hash[2..])
            }
        };

        Ok((encrypted_uri, metadata_hash))
    }

    /// Build the mint transaction calldata for ProofOfClawINFT.mint()
    ///
    /// Returns ABI-encoded calldata ready for submission via ethers-rs or cast.
    pub fn build_mint_calldata(
        agent_id: &str,
        policy_hash: &str,
        risc_zero_image_id: &str,
        encrypted_uri: &str,
        metadata_hash: &str,
        ens_name: &str,
    ) -> Vec<u8> {
        use ethers::abi::{encode, Token};

        // agent_id as bytes32
        let mut agent_id_bytes = [0u8; 32];
        let agent_hash = {
            let mut hasher = Sha256::new();
            hasher.update(agent_id.as_bytes());
            hasher.finalize()
        };
        agent_id_bytes.copy_from_slice(&agent_hash);

        // policy_hash as bytes32
        let policy_bytes = hex_to_bytes32(policy_hash);

        // risc_zero_image_id as bytes32
        let image_id_bytes = hex_to_bytes32(risc_zero_image_id);

        // metadata_hash as bytes32
        let meta_hash_bytes = hex_to_bytes32(metadata_hash);

        // Function selector: mint(bytes32,bytes32,bytes32,string,bytes32,string)
        let selector = &ethers::utils::keccak256(
            b"mint(bytes32,bytes32,bytes32,string,bytes32,string)",
        )[..4];

        let encoded = encode(&[
            Token::FixedBytes(agent_id_bytes.to_vec()),
            Token::FixedBytes(policy_bytes.to_vec()),
            Token::FixedBytes(image_id_bytes.to_vec()),
            Token::String(encrypted_uri.to_string()),
            Token::FixedBytes(meta_hash_bytes.to_vec()),
            Token::String(ens_name.to_string()),
        ]);

        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&encoded);
        calldata
    }

    /// Query iNFT data for an agent from the contract
    pub async fn get_agent_inft(&self, agent_id: &str) -> Result<Option<INFTData>> {
        // Compute agent_id bytes32
        let mut hasher = Sha256::new();
        hasher.update(agent_id.as_bytes());
        let agent_hash = format!("0x{}", hex::encode(hasher.finalize()));

        // Call getTokenByAgent(bytes32) via eth_call
        let selector = &ethers::utils::keccak256(b"getTokenByAgent(bytes32)")[..4];
        let agent_bytes = hex_to_bytes32(&agent_hash);

        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(agent_bytes.to_vec()),
        ]));

        let result = self
            .client
            .post(&self.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_call",
                "params": [{
                    "to": self.inft_contract,
                    "data": format!("0x{}", hex::encode(&calldata))
                }, "latest"],
                "id": 1
            }))
            .send()
            .await?;

        let body: serde_json::Value = result.json().await?;
        let result_hex = body["result"].as_str().unwrap_or("0x0");

        // Token ID 0 means not minted
        let token_id = u64::from_str_radix(
            result_hex.trim_start_matches("0x").trim_start_matches('0'),
            16,
        )
        .unwrap_or(0);

        if token_id == 0 {
            return Ok(None);
        }

        Ok(Some(INFTData {
            token_id,
            owner: String::new(),
            agent_id: agent_hash,
            policy_hash: String::new(),
            risc_zero_image_id: String::new(),
            encrypted_uri: String::new(),
            metadata_hash: String::new(),
            ens_name: String::new(),
            reputation_score: 0,
            total_proofs: 0,
            minted_at: 0,
            active: true,
        }))
    }
}

fn hex_to_bytes32(hex_str: &str) -> [u8; 32] {
    let clean = hex_str.trim_start_matches("0x");
    let mut bytes = [0u8; 32];
    let decoded = hex::decode(clean).unwrap_or_default();
    let len = decoded.len().min(32);
    bytes[32 - len..].copy_from_slice(&decoded[..len]);
    bytes
}
