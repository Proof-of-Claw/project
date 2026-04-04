use crate::core::config::PolicyConfig;
use crate::core::types::{AgentMessage, InferenceResponse, PolicyResult, PolicySeverity};
use anyhow::Result;

pub struct PolicyEngine {
    config: PolicyConfig,
}

impl PolicyEngine {
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }
    
    pub fn check(&self, message: &AgentMessage, _inference: &InferenceResponse) -> Result<PolicyResult> {
        let action = &message.payload.action;
        
        if !self.config.allowed_tools.contains(action) {
            return Ok(PolicyResult {
                rule_id: "tool_allowlist".to_string(),
                severity: PolicySeverity::Block,
                details: format!("Tool '{}' not in allowed list", action),
            });
        }
        
        if let Some(value) = message.payload.params.get("value") {
            if let Some(value_u64) = value.as_u64() {
                if value_u64 > self.config.max_value_autonomous_wei {
                    return Ok(PolicyResult {
                        rule_id: "value_threshold".to_string(),
                        severity: PolicySeverity::Warn,
                        details: format!("Value {} exceeds autonomous threshold", value_u64),
                    });
                }
            }
        }
        
        Ok(PolicyResult {
            rule_id: "default".to_string(),
            severity: PolicySeverity::Pass,
            details: "All checks passed".to_string(),
        })
    }
}
