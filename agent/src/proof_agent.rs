use anyhow::Result;
use tracing::{info, warn};
use crate::core::config::AgentConfig;
use crate::integrations::zero_g::{ZeroGCompute, ZeroGStorage};
use crate::integrations::ens_dm3::DM3Client;
use crate::ironclaw_adapter::{IronClawAdapter, IronClawExecutionTrace};

pub struct ProofOfClawAgent {
    config: AgentConfig,
    adapter: IronClawAdapter,
}

impl ProofOfClawAgent {
    pub async fn new(config: AgentConfig) -> Result<Self> {
        info!("Initializing Proof of Claw Agent (IronClaw-based): {}", config.agent_id);
        
        let zero_g_compute = ZeroGCompute::new(&config).await?;
        let zero_g_storage = ZeroGStorage::new(&config).await?;
        let dm3_client = DM3Client::new(&config).await?;
        
        let adapter = IronClawAdapter::new(zero_g_compute, zero_g_storage, dm3_client);
        
        Ok(Self { config, adapter })
    }

    pub fn id(&self) -> &str {
        &self.config.agent_id
    }

    #[cfg(feature = "ironclaw-integration")]
    pub async fn run_with_ironclaw(&self) -> Result<()> {
        info!("Starting Proof of Claw Agent with IronClaw runtime");
        
        warn!("IronClaw integration requires the full IronClaw runtime to be initialized.");
        warn!("This is a placeholder for the full integration.");
        warn!("In production, this would:");
        warn!("  1. Initialize IronClaw's Agent with our hooks");
        warn!("  2. Register our 0G Compute as the LLM provider");
        warn!("  3. Intercept all tool executions for trace generation");
        warn!("  4. Store execution traces on 0G Storage");
        warn!("  5. Generate RISC Zero proofs from IronClaw traces");
        
        Ok(())
    }

    pub async fn run_standalone(&mut self) -> Result<()> {
        info!("Starting Proof of Claw Agent in standalone mode");
        
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutting down agent");
                    break;
                }
            }
        }
        
        Ok(())
    }

    pub async fn process_ironclaw_trace(&self, trace: IronClawExecutionTrace) -> Result<String> {
        info!("Processing IronClaw execution trace");
        
        let proof_trace = self.adapter.convert_trace(trace, &self.config.agent_id);
        
        let trace_hash = self.adapter.store_trace(&proof_trace).await?;
        info!("Trace stored on 0G Storage: {}", trace_hash);
        
        Ok(trace_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ironclaw_adapter::{IronClawToolCall, LLMInteraction, PolicyCheck};

    fn create_test_config() -> AgentConfig {
        AgentConfig {
            agent_id: "test-agent".to_string(),
            ens_name: "test.proofclaw.eth".to_string(),
            private_key: "0x1234".to_string(),
            rpc_url: "http://localhost:8545".to_string(),
            zero_g_indexer_rpc: "http://localhost:5000".to_string(),
            zero_g_compute_endpoint: "http://localhost:5001".to_string(),
            dm3_delivery_service_url: "http://localhost:3001".to_string(),
            ledger_origin_token: None,
            policy: crate::core::config::PolicyConfig {
                allowed_tools: vec!["swap".to_string()],
                endpoint_allowlist: vec!["https://api.example.com".to_string()],
                max_value_autonomous_wei: 1_000_000_000_000_000_000,
            },
        }
    }

    #[tokio::test]
    async fn test_ironclaw_trace_conversion() {
        let config = create_test_config();
        let agent = ProofOfClawAgent::new(config).await.unwrap();

        let ironclaw_trace = IronClawExecutionTrace {
            session_id: "test-session".to_string(),
            timestamp: 1234567890,
            tool_calls: vec![
                IronClawToolCall {
                    tool_name: "swap".to_string(),
                    input: serde_json::json!({"amount": 100}),
                    output: serde_json::json!({"success": true}),
                    capability_hash: "0xabcd".to_string(),
                    allowed: true,
                }
            ],
            llm_interactions: vec![
                LLMInteraction {
                    prompt: "Swap 100 tokens".to_string(),
                    response: "Executing swap".to_string(),
                    provider: "0g-compute".to_string(),
                    attestation: Some("0x1234567890".to_string()),
                }
            ],
            policy_checks: vec![
                PolicyCheck {
                    rule_id: "tool_allowlist".to_string(),
                    severity: "pass".to_string(),
                    passed: true,
                    details: "Tool is allowed".to_string(),
                }
            ],
        };

        let proof_trace = agent.adapter.convert_trace(ironclaw_trace, "test-agent");
        
        assert_eq!(proof_trace.agent_id, "test-agent");
        assert_eq!(proof_trace.tool_invocations.len(), 1);
        assert_eq!(proof_trace.tool_invocations[0].tool_name, "swap");
        assert_eq!(proof_trace.policy_check_results.len(), 1);
    }
}
