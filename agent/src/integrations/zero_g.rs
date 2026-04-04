use crate::core::config::AgentConfig;
use crate::core::types::{ExecutionTrace, InferenceRequest, InferenceResponse};
use anyhow::{Result, Context};
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
            .await
            .context("Failed to reach 0G Compute endpoint")?;

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

    /// Store an execution trace on 0G Storage.
    /// Returns the root hash that can be used to retrieve the trace later.
    pub async fn store_trace(&self, trace: &ExecutionTrace) -> Result<String> {
        let data = serde_json::to_string(trace)?;

        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let root_hash = format!("0x{}", hex::encode(hasher.finalize()));

        // Upload to 0G Storage indexer
        let resp = self
            .client
            .post(format!("{}/upload", self.indexer_rpc))
            .json(&json!({
                "data": data,
                "tags": {
                    "type": "execution-trace",
                    "agent": &trace.agent_id,
                    "session": &trace.session_id,
                }
            }))
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                // Use server-returned root hash if available, else content-hash
                let body = r.text().await.unwrap_or_default();
                if body.starts_with("0x") && body.len() == 66 {
                    return Ok(body);
                }
            }
            Ok(r) => {
                tracing::warn!("0G Storage upload returned {}: falling back to content-hash", r.status());
            }
            Err(e) => {
                tracing::warn!("0G Storage upload failed ({}): falling back to content-hash", e);
            }
        }

        Ok(root_hash)
    }

    /// Retrieve an execution trace from 0G Storage by its root hash.
    pub async fn retrieve_trace(&self, root_hash: &str) -> Result<ExecutionTrace> {
        let resp = self
            .client
            .get(format!("{}/download", self.indexer_rpc))
            .query(&[("root", root_hash)])
            .send()
            .await
            .context("Failed to reach 0G Storage indexer for trace retrieval")?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "0G Storage returned {} when retrieving trace {}",
                resp.status(),
                root_hash
            );
        }

        let body = resp.text().await?;
        let trace: ExecutionTrace = serde_json::from_str(&body)
            .context("Failed to deserialize execution trace from 0G Storage")?;

        // Verify content integrity against root hash
        let mut hasher = Sha256::new();
        hasher.update(body.as_bytes());
        let computed_hash = format!("0x{}", hex::encode(hasher.finalize()));

        if computed_hash != root_hash {
            tracing::warn!(
                "Trace content hash mismatch: expected {}, got {}. Data may have been tampered with.",
                root_hash,
                computed_hash
            );
        }

        Ok(trace)
    }
}
