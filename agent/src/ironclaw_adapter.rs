use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::core::types::ExecutionTrace as ProofOfClawTrace;
use crate::integrations::zero_g::{ZeroGCompute, ZeroGStorage};
use crate::integrations::ens_dm3::DM3Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronClawExecutionTrace {
    pub session_id: String,
    pub timestamp: i64,
    pub tool_calls: Vec<IronClawToolCall>,
    pub llm_interactions: Vec<LLMInteraction>,
    pub policy_checks: Vec<PolicyCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronClawToolCall {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub capability_hash: String,
    pub allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMInteraction {
    pub prompt: String,
    pub response: String,
    pub provider: String,
    pub attestation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheck {
    pub rule_id: String,
    pub severity: String,
    pub passed: bool,
    pub details: String,
}

pub struct IronClawAdapter {
    zero_g_compute: ZeroGCompute,
    zero_g_storage: ZeroGStorage,
    dm3_client: DM3Client,
}

impl IronClawAdapter {
    pub fn new(
        zero_g_compute: ZeroGCompute,
        zero_g_storage: ZeroGStorage,
        dm3_client: DM3Client,
    ) -> Self {
        Self {
            zero_g_compute,
            zero_g_storage,
            dm3_client,
        }
    }

    pub fn convert_trace(&self, ironclaw_trace: IronClawExecutionTrace, agent_id: &str) -> ProofOfClawTrace {
        use sha2::{Sha256, Digest};
        
        let tool_invocations = ironclaw_trace.tool_calls.iter().map(|call| {
            let input_hash = {
                let mut hasher = Sha256::new();
                hasher.update(serde_json::to_string(&call.input).unwrap().as_bytes());
                format!("0x{}", hex::encode(hasher.finalize()))
            };
            
            let output_hash = {
                let mut hasher = Sha256::new();
                hasher.update(serde_json::to_string(&call.output).unwrap().as_bytes());
                format!("0x{}", hex::encode(hasher.finalize()))
            };
            
            crate::core::types::ToolInvocation {
                tool_name: call.tool_name.clone(),
                input_hash,
                output_hash,
                capability_hash: call.capability_hash.clone(),
                timestamp: ironclaw_trace.timestamp,
                within_policy: call.allowed,
            }
        }).collect();

        let policy_check_results = ironclaw_trace.policy_checks.iter().map(|check| {
            crate::core::types::PolicyResult {
                rule_id: check.rule_id.clone(),
                severity: match check.severity.as_str() {
                    "block" => crate::core::types::PolicySeverity::Block,
                    "warn" => crate::core::types::PolicySeverity::Warn,
                    "sanitize" => crate::core::types::PolicySeverity::Sanitize,
                    _ => crate::core::types::PolicySeverity::Pass,
                },
                details: check.details.clone(),
            }
        }).collect();

        let inference_commitment = ironclaw_trace.llm_interactions
            .first()
            .and_then(|i| i.attestation.clone())
            .unwrap_or_else(|| {
                tracing::warn!("LLM interaction has no attestation — inference may not be verifiable");
                String::new()
            });

        let output_commitment = {
            let mut hasher = Sha256::new();
            hasher.update(serde_json::to_string(&ironclaw_trace).unwrap().as_bytes());
            format!("0x{}", hex::encode(hasher.finalize()))
        };

        ProofOfClawTrace {
            agent_id: agent_id.to_string(),
            session_id: ironclaw_trace.session_id,
            timestamp: ironclaw_trace.timestamp,
            inference_commitment,
            tool_invocations,
            policy_check_results,
            output_commitment,
        }
    }

    pub async fn intercept_llm_call(&self, prompt: &str) -> Result<String> {
        let inference_request = crate::core::types::InferenceRequest {
            system_prompt: "You are a secure AI agent running in IronClaw with Proof of Claw verification.".to_string(),
            user_prompt: prompt.to_string(),
            model: None,
        };

        let response = self.zero_g_compute.inference(&inference_request).await?;
        Ok(response.content)
    }

    pub async fn store_trace(&self, trace: &ProofOfClawTrace) -> Result<String> {
        self.zero_g_storage.store_trace(trace).await
    }

    pub async fn send_agent_message(&self, recipient: &str, message: &crate::core::types::AgentMessage) -> Result<()> {
        self.dm3_client.send_message(recipient, message).await
    }
}

#[cfg(feature = "ironclaw-integration")]
pub mod ironclaw_hooks {
    use super::*;

    pub struct ProofOfClawHooks {
        adapter: IronClawAdapter,
    }

    impl ProofOfClawHooks {
        pub fn new(adapter: IronClawAdapter) -> Self {
            Self { adapter }
        }

        pub async fn on_tool_execution(
            &self,
            tool_name: &str,
            input: &serde_json::Value,
            output: &serde_json::Value,
        ) -> Result<()> {
            tracing::info!("Tool executed: {} (Proof of Claw tracking)", tool_name);
            Ok(())
        }

        pub async fn on_llm_call(&self, prompt: &str) -> Result<String> {
            self.adapter.intercept_llm_call(prompt).await
        }

        pub async fn on_session_complete(&self, trace: IronClawExecutionTrace, agent_id: &str) -> Result<()> {
            let proof_trace = self.adapter.convert_trace(trace, agent_id);
            let trace_hash = self.adapter.store_trace(&proof_trace).await?;
            tracing::info!("Session trace stored on 0G Storage: {}", trace_hash);
            Ok(())
        }
    }
}
