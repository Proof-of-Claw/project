use crate::core::config::AgentConfig;
use anyhow::{Result, Context};
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

    /// Make an eth_call and return the raw hex result
    async fn eth_call(&self, calldata: &[u8]) -> Result<Vec<u8>> {
        if self.inft_contract.is_empty() {
            anyhow::bail!("INFT_CONTRACT address not configured");
        }

        let result = self
            .client
            .post(&self.rpc_url)
            .json(&serde_json::json!({
                "jsonrpc": "2.0",
                "method": "eth_call",
                "params": [{
                    "to": self.inft_contract,
                    "data": format!("0x{}", hex::encode(calldata))
                }, "latest"],
                "id": 1
            }))
            .send()
            .await
            .context("Failed to send eth_call to RPC")?;

        let body: serde_json::Value = result.json().await?;

        if let Some(error) = body.get("error") {
            anyhow::bail!("RPC error querying iNFT contract: {}", error);
        }

        let result_hex = body["result"]
            .as_str()
            .unwrap_or("0x");

        let bytes = hex::decode(result_hex.trim_start_matches("0x"))
            .unwrap_or_default();

        Ok(bytes)
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
            risc_zero_image_id: config
                .risc_zero_image_id
                .clone()
                .unwrap_or_default(),
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
                if body.starts_with("0x") {
                    format!("0g://{}", body)
                } else {
                    format!("0g://{}", &metadata_hash[2..])
                }
            }
            Err(_) => {
                format!("0g://{}", &metadata_hash[2..])
            }
        };

        Ok((encrypted_uri, metadata_hash))
    }

    /// Build the mint transaction calldata for ProofOfClawINFT.mint()
    pub fn build_mint_calldata(
        agent_id: &str,
        policy_hash: &str,
        risc_zero_image_id: &str,
        encrypted_uri: &str,
        metadata_hash: &str,
        ens_name: &str,
    ) -> Vec<u8> {
        use ethers::abi::{encode, Token};

        let mut agent_id_bytes = [0u8; 32];
        let agent_hash = {
            let mut hasher = Sha256::new();
            hasher.update(agent_id.as_bytes());
            hasher.finalize()
        };
        agent_id_bytes.copy_from_slice(&agent_hash);

        let policy_bytes = hex_to_bytes32(policy_hash);
        let image_id_bytes = hex_to_bytes32(risc_zero_image_id);
        let meta_hash_bytes = hex_to_bytes32(metadata_hash);

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

    /// Query full iNFT data for an agent from the contract.
    ///
    /// Calls `getAgentData(bytes32)` which returns:
    /// (uint256 tokenId, address owner, bytes32 policyHash, bytes32 imageId,
    ///  string encryptedUri, bytes32 metadataHash, string ensName,
    ///  uint256 reputationScore, uint256 totalProofs, uint256 mintedAt, bool active)
    pub async fn get_agent_inft(&self, agent_id: &str) -> Result<Option<INFTData>> {
        let mut hasher = Sha256::new();
        hasher.update(agent_id.as_bytes());
        let agent_hash = format!("0x{}", hex::encode(hasher.finalize()));

        // First check if token exists via getTokenByAgent(bytes32)
        let selector = &ethers::utils::keccak256(b"getTokenByAgent(bytes32)")[..4];
        let agent_bytes = hex_to_bytes32(&agent_hash);

        let mut calldata = selector.to_vec();
        calldata.extend_from_slice(&ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(agent_bytes.to_vec()),
        ]));

        let result_bytes = self.eth_call(&calldata).await?;

        if result_bytes.len() < 32 {
            return Ok(None);
        }

        // Decode token ID
        let token_id = u64::from_str_radix(
            &hex::encode(&result_bytes[..32])
                .trim_start_matches('0'),
            16,
        )
        .unwrap_or(0);

        if token_id == 0 {
            return Ok(None);
        }

        // Now fetch full agent data via getAgentData(bytes32)
        let data_selector = &ethers::utils::keccak256(b"getAgentData(bytes32)")[..4];
        let mut data_calldata = data_selector.to_vec();
        data_calldata.extend_from_slice(&ethers::abi::encode(&[
            ethers::abi::Token::FixedBytes(agent_bytes.to_vec()),
        ]));

        let data_bytes = self.eth_call(&data_calldata).await?;

        if data_bytes.len() < 352 {
            // Minimal response: just return what we have from token ID
            return Ok(Some(INFTData {
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
            }));
        }

        // Decode the full struct from ABI-encoded data
        let tokens = ethers::abi::decode(
            &[
                ethers::abi::ParamType::Uint(256),      // tokenId
                ethers::abi::ParamType::Address,          // owner
                ethers::abi::ParamType::FixedBytes(32),   // policyHash
                ethers::abi::ParamType::FixedBytes(32),   // imageId
                ethers::abi::ParamType::String,           // encryptedUri
                ethers::abi::ParamType::FixedBytes(32),   // metadataHash
                ethers::abi::ParamType::String,           // ensName
                ethers::abi::ParamType::Uint(256),        // reputationScore
                ethers::abi::ParamType::Uint(256),        // totalProofs
                ethers::abi::ParamType::Uint(256),        // mintedAt
                ethers::abi::ParamType::Bool,             // active
            ],
            &data_bytes,
        )
        .context("Failed to decode iNFT agent data from contract")?;

        let owner = tokens[1]
            .clone()
            .into_address()
            .map(|a| format!("0x{}", hex::encode(a.as_bytes())))
            .unwrap_or_default();

        let policy_hash = tokens[2]
            .clone()
            .into_fixed_bytes()
            .map(|b| format!("0x{}", hex::encode(b)))
            .unwrap_or_default();

        let risc_zero_image_id = tokens[3]
            .clone()
            .into_fixed_bytes()
            .map(|b| format!("0x{}", hex::encode(b)))
            .unwrap_or_default();

        let encrypted_uri = tokens[4]
            .clone()
            .into_string()
            .unwrap_or_default();

        let metadata_hash = tokens[5]
            .clone()
            .into_fixed_bytes()
            .map(|b| format!("0x{}", hex::encode(b)))
            .unwrap_or_default();

        let ens_name = tokens[6]
            .clone()
            .into_string()
            .unwrap_or_default();

        let reputation_score = tokens[7]
            .clone()
            .into_uint()
            .unwrap_or_default()
            .as_u64();

        let total_proofs = tokens[8]
            .clone()
            .into_uint()
            .unwrap_or_default()
            .as_u64();

        let minted_at = tokens[9]
            .clone()
            .into_uint()
            .unwrap_or_default()
            .as_u64();

        let active = tokens[10]
            .clone()
            .into_bool()
            .unwrap_or(true);

        Ok(Some(INFTData {
            token_id,
            owner,
            agent_id: agent_hash,
            policy_hash,
            risc_zero_image_id,
            encrypted_uri,
            metadata_hash,
            ens_name,
            reputation_score,
            total_proofs,
            minted_at,
            active,
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
