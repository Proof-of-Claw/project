#[cfg(test)]
mod tests {
    use super::super::policy_engine::PolicyEngine;
    use crate::core::config::PolicyConfig;
    use crate::core::types::{AgentMessage, MessageType, MessagePayload, InferenceResponse, PolicySeverity};
    use std::collections::HashMap;

    fn create_test_policy() -> PolicyConfig {
        PolicyConfig {
            allowed_tools: vec!["swap_tokens".to_string(), "transfer".to_string()],
            endpoint_allowlist: vec!["https://api.uniswap.org".to_string()],
            max_value_autonomous_wei: 1_000_000_000_000_000_000,
        }
    }

    fn create_test_message(action: &str, value: Option<u64>) -> AgentMessage {
        let mut params = HashMap::new();
        if let Some(v) = value {
            params.insert("value".to_string(), serde_json::json!(v));
        }

        AgentMessage {
            message_type: MessageType::Propose,
            payload: MessagePayload {
                action: action.to_string(),
                params,
                trace_root_hash: None,
                proof_receipt: None,
                required_approval: None,
            },
            nonce: 1,
            timestamp: 0,
        }
    }

    fn create_test_inference() -> InferenceResponse {
        InferenceResponse {
            content: "test response".to_string(),
            attestation_signature: "0x1234".to_string(),
            provider: "test".to_string(),
        }
    }

    #[test]
    fn test_allowed_tool_passes() {
        let policy = create_test_policy();
        let engine = PolicyEngine::new(policy);
        let message = create_test_message("swap_tokens", None);
        let inference = create_test_inference();

        let result = engine.check(&message, &inference).unwrap();
        assert!(matches!(result.severity, PolicySeverity::Pass));
    }

    #[test]
    fn test_disallowed_tool_blocks() {
        let policy = create_test_policy();
        let engine = PolicyEngine::new(policy);
        let message = create_test_message("malicious_action", None);
        let inference = create_test_inference();

        let result = engine.check(&message, &inference).unwrap();
        assert!(matches!(result.severity, PolicySeverity::Block));
        assert_eq!(result.rule_id, "tool_allowlist");
    }

    #[test]
    fn test_value_threshold_warning() {
        let policy = create_test_policy();
        let engine = PolicyEngine::new(policy);
        let message = create_test_message("swap_tokens", Some(2_000_000_000_000_000_000));
        let inference = create_test_inference();

        let result = engine.check(&message, &inference).unwrap();
        assert!(matches!(result.severity, PolicySeverity::Warn));
        assert_eq!(result.rule_id, "value_threshold");
    }

    #[test]
    fn test_value_below_threshold_passes() {
        let policy = create_test_policy();
        let engine = PolicyEngine::new(policy);
        let message = create_test_message("swap_tokens", Some(500_000_000_000_000_000));
        let inference = create_test_inference();

        let result = engine.check(&message, &inference).unwrap();
        assert!(matches!(result.severity, PolicySeverity::Pass));
    }
}
