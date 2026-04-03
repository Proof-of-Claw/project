use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    pub agent_id: String,
    pub session_id: String,
    pub timestamp: i64,
    pub inference_commitment: String,
    pub tool_invocations: Vec<ToolInvocation>,
    pub policy_check_results: Vec<PolicyResult>,
    pub output_commitment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub tool_name: String,
    pub input_hash: String,
    pub output_hash: String,
    pub capability_hash: String,
    pub timestamp: i64,
    pub within_policy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub rule_id: String,
    pub severity: PolicySeverity,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicySeverity {
    Block,
    Warn,
    Sanitize,
    Pass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPolicy {
    pub allowed_tools: Vec<String>,
    pub endpoint_allowlist: Vec<String>,
    pub max_value_autonomous: u64,
    pub capability_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub message_type: MessageType,
    pub payload: MessagePayload,
    pub nonce: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Propose,
    Accept,
    Reject,
    Execute,
    Verify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub action: String,
    pub params: HashMap<String, serde_json::Value>,
    pub trace_root_hash: Option<String>,
    pub proof_receipt: Option<String>,
    pub required_approval: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceRequest {
    pub system_prompt: String,
    pub user_prompt: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResponse {
    pub content: String,
    pub attestation_signature: String,
    pub provider: String,
}
