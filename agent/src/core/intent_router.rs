use super::types::AgentMessage;
use anyhow::Result;

#[derive(Debug, Clone)]
pub enum Intent {
    TokenSwap,
    Transfer,
    Query,
    Negotiate,
    Unknown,
}

pub struct IntentRouter;

impl IntentRouter {
    pub fn new() -> Self {
        Self
    }
    
    pub fn classify_intent(&self, message: &AgentMessage) -> Result<Intent> {
        let action = &message.payload.action;
        
        let intent = match action.as_str() {
            "swap_tokens" => Intent::TokenSwap,
            "transfer" => Intent::Transfer,
            "query" => Intent::Query,
            "negotiate" => Intent::Negotiate,
            _ => Intent::Unknown,
        };
        
        Ok(intent)
    }
}
