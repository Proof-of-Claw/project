use anyhow::Result;

pub struct LedgerApprovalGate {
    origin_token: Option<String>,
}

impl LedgerApprovalGate {
    pub fn new(origin_token: Option<String>) -> Self {
        Self { origin_token }
    }
    
    pub async fn request_approval(&self, _action_description: &str, _value: u64) -> Result<bool> {
        Ok(true)
    }
}
