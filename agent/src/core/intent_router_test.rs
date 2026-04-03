#[cfg(test)]
mod tests {
    use super::super::intent_router::{Intent, IntentRouter};
    use super::super::types::{AgentMessage, MessageType, MessagePayload};
    use std::collections::HashMap;

    fn create_test_message(action: &str) -> AgentMessage {
        AgentMessage {
            message_type: MessageType::Propose,
            payload: MessagePayload {
                action: action.to_string(),
                params: HashMap::new(),
                trace_root_hash: None,
                proof_receipt: None,
                required_approval: None,
            },
            nonce: 1,
            timestamp: 0,
        }
    }

    #[test]
    fn test_classify_swap_intent() {
        let router = IntentRouter::new();
        let message = create_test_message("swap_tokens");
        
        let intent = router.classify_intent(&message).unwrap();
        assert!(matches!(intent, Intent::TokenSwap));
    }

    #[test]
    fn test_classify_transfer_intent() {
        let router = IntentRouter::new();
        let message = create_test_message("transfer");
        
        let intent = router.classify_intent(&message).unwrap();
        assert!(matches!(intent, Intent::Transfer));
    }

    #[test]
    fn test_classify_query_intent() {
        let router = IntentRouter::new();
        let message = create_test_message("query");
        
        let intent = router.classify_intent(&message).unwrap();
        assert!(matches!(intent, Intent::Query));
    }

    #[test]
    fn test_classify_unknown_intent() {
        let router = IntentRouter::new();
        let message = create_test_message("unknown_action");
        
        let intent = router.classify_intent(&message).unwrap();
        assert!(matches!(intent, Intent::Unknown));
    }
}
