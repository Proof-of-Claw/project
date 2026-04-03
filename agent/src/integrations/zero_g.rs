use crate::core::config::AgentConfig;
use crate::core::types::{ExecutionTrace, InferenceRequest, InferenceResponse};
use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use sha2::{Sha256, Digest};

pub struct ZeroGCompute {
    client: Client,
    endpoint: String,
}

impl ZeroGCompute {
    pub async fn new(config: &AgentConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            endpoint: config.zero_g_compute_endpoint.clone(),
        })
    }
    
    pub async fn inference(&self, request: &InferenceRequest) -> Result<InferenceResponse> {
        let response = self.client
            .post(format!("{}/chat/completions", self.endpoint))
            .json(&json!({
                "messages": [
                    {"role": "system", "content": request.system_prompt},
                    {"role": "user", "content": request.user_prompt}
                ],
                "model": request.model.as_ref().unwrap_or(&"gpt-4".to_string())
            }))
            .send()
            .await?;
        
        let body = response.text().await?;
        
        let mut hasher = Sha256::new();
        hasher.update(body.as_bytes());
        let attestation = format!("0x{}", hex::encode(hasher.finalize()));
        
        Ok(InferenceResponse {
            content: body,
            attestation_signature: attestation,
            provider: "0g-compute".to_string(),
        })
    }
}

pub struct ZeroGStorage {
    client: Client,
    indexer_rpc: String,
}

impl ZeroGStorage {
    pub async fn new(config: &AgentConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            indexer_rpc: config.zero_g_indexer_rpc.clone(),
        })
    }
    
    pub async fn store_trace(&self, trace: &ExecutionTrace) -> Result<String> {
        let data = serde_json::to_string(trace)?;
        
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let root_hash = format!("0x{}", hex::encode(hasher.finalize()));
        
        Ok(root_hash)
    }
    
    pub async fn retrieve_trace(&self, root_hash: &str) -> Result<ExecutionTrace> {
        let trace = ExecutionTrace {
            agent_id: "placeholder".to_string(),
            session_id: "placeholder".to_string(),
            timestamp: 0,
            inference_commitment: String::new(),
            tool_invocations: Vec::new(),
            policy_check_results: Vec::new(),
            output_commitment: String::new(),
        };
        
        Ok(trace)
    }
}
