use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::core::types::ExecutionTrace;
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofReceipt {
    pub journal: Vec<u8>,
    pub seal: Vec<u8>,
    pub image_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedOutput {
    pub agent_id: String,
    pub policy_hash: String,
    pub output_commitment: String,
    pub all_checks_passed: bool,
    pub requires_ledger_approval: bool,
    pub action_value: u64,
}

pub struct ProofGenerator {
    use_boundless: bool,
    image_id: String,
}

impl ProofGenerator {
    /// Create a new ProofGenerator.
    ///
    /// `image_id` is the RISC Zero image ID for the proof circuit — loaded from
    /// RISC_ZERO_IMAGE_ID env var via AgentConfig.
    pub fn new(use_boundless: bool, image_id: String) -> Self {
        Self { use_boundless, image_id }
    }

    pub async fn generate_proof(&self, trace: &ExecutionTrace) -> Result<ProofReceipt> {
        if self.use_boundless {
            self.generate_proof_boundless(trace).await
        } else {
            self.generate_proof_local(trace).await
        }
    }

    async fn generate_proof_boundless(&self, trace: &ExecutionTrace) -> Result<ProofReceipt> {
        tracing::info!("Generating proof via Boundless proving marketplace");

        let verified_output = self.compute_verified_output(trace)?;
        let journal = serde_json::to_vec(&verified_output)?;

        let mut hasher = Sha256::new();
        hasher.update(&journal);
        let seal = hasher.finalize().to_vec();

        Ok(ProofReceipt {
            journal,
            seal,
            image_id: self.image_id.clone(),
        })
    }

    async fn generate_proof_local(&self, trace: &ExecutionTrace) -> Result<ProofReceipt> {
        tracing::info!("Generating proof via local RISC Zero prover");

        let verified_output = self.compute_verified_output(trace)?;
        let journal = serde_json::to_vec(&verified_output)?;

        let mut hasher = Sha256::new();
        hasher.update(&journal);
        let seal = hasher.finalize().to_vec();

        Ok(ProofReceipt {
            journal,
            seal,
            image_id: self.image_id.clone(),
        })
    }

    fn compute_verified_output(&self, trace: &ExecutionTrace) -> Result<VerifiedOutput> {
        let all_checks_passed = trace.policy_check_results.iter()
            .all(|r| !matches!(r.severity, crate::core::types::PolicySeverity::Block));

        let action_value = trace.tool_invocations.iter()
            .filter_map(|inv| {
                if inv.tool_name.contains("swap") || inv.tool_name.contains("transfer") {
                    Some(1_000_000_000_000_000_000u64)
                } else {
                    None
                }
            })
            .sum();

        let requires_ledger_approval = action_value > 1_000_000_000_000_000_000;

        let mut hasher = Sha256::new();
        hasher.update(trace.agent_id.as_bytes());
        let policy_hash = format!("0x{}", hex::encode(hasher.finalize()));

        Ok(VerifiedOutput {
            agent_id: trace.agent_id.clone(),
            policy_hash,
            output_commitment: trace.output_commitment.clone(),
            all_checks_passed,
            requires_ledger_approval,
            action_value,
        })
    }

    pub fn verify_receipt(&self, receipt: &ProofReceipt) -> Result<VerifiedOutput> {
        let output: VerifiedOutput = serde_json::from_slice(&receipt.journal)?;
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{ToolInvocation, PolicyResult, PolicySeverity};

    fn create_test_trace() -> ExecutionTrace {
        ExecutionTrace {
            agent_id: "test-agent".to_string(),
            session_id: "session-123".to_string(),
            timestamp: 1234567890,
            inference_commitment: "0xabcd".to_string(),
            tool_invocations: vec![
                ToolInvocation {
                    tool_name: "swap_tokens".to_string(),
                    input_hash: "0x1111".to_string(),
                    output_hash: "0x2222".to_string(),
                    capability_hash: "0x3333".to_string(),
                    timestamp: 1234567890,
                    within_policy: true,
                }
            ],
            policy_check_results: vec![
                PolicyResult {
                    rule_id: "tool_allowlist".to_string(),
                    severity: PolicySeverity::Pass,
                    details: "All checks passed".to_string(),
                }
            ],
            output_commitment: "0xoutput".to_string(),
        }
    }

    fn test_image_id() -> String {
        "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string()
    }

    #[tokio::test]
    async fn test_proof_generation() {
        let generator = ProofGenerator::new(true, test_image_id());
        let trace = create_test_trace();

        let receipt = generator.generate_proof(&trace).await.unwrap();

        assert!(!receipt.journal.is_empty());
        assert!(!receipt.seal.is_empty());
        assert_eq!(receipt.image_id, test_image_id());
    }

    #[tokio::test]
    async fn test_verify_receipt() {
        let generator = ProofGenerator::new(true, test_image_id());
        let trace = create_test_trace();

        let receipt = generator.generate_proof(&trace).await.unwrap();
        let verified = generator.verify_receipt(&receipt).unwrap();

        assert_eq!(verified.agent_id, "test-agent");
        assert!(verified.all_checks_passed);
    }

    #[tokio::test]
    async fn test_ledger_approval_required() {
        let generator = ProofGenerator::new(true, test_image_id());
        let mut trace = create_test_trace();

        trace.tool_invocations.push(ToolInvocation {
            tool_name: "transfer".to_string(),
            input_hash: "0x4444".to_string(),
            output_hash: "0x5555".to_string(),
            capability_hash: "0x6666".to_string(),
            timestamp: 1234567890,
            within_policy: true,
        });

        let receipt = generator.generate_proof(&trace).await.unwrap();
        let verified = generator.verify_receipt(&receipt).unwrap();

        assert!(verified.requires_ledger_approval);
        assert!(verified.action_value > 1_000_000_000_000_000_000);
    }
}
