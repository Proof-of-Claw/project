use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesFile {
    pub allowed_endpoints: Vec<String>,
    pub required_secrets: Vec<String>,
    pub rate_limits: RateLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub max_value_per_request: u64,
}

impl CapabilitiesFile {
    pub fn validate_endpoint(&self, endpoint: &str) -> bool {
        self.allowed_endpoints.iter().any(|allowed| endpoint.starts_with(allowed))
    }
}
