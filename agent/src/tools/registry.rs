use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub capability_hash: String,
    pub wasm_module: Option<Vec<u8>>,
}

pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

/// Compute a deterministic capability hash for a tool from its name, description,
/// and optional WASM bytecode. This produces a content-addressable identifier
/// that changes when the tool's behavior changes.
fn compute_capability_hash(name: &str, description: &str, wasm_module: &Option<Vec<u8>>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(name.as_bytes());
    hasher.update(b"|");
    hasher.update(description.as_bytes());
    if let Some(wasm) = wasm_module {
        hasher.update(b"|");
        hasher.update(wasm);
    }
    format!("0x{}", hex::encode(hasher.finalize()))
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        registry.register_builtin_tools();
        registry
    }

    fn register_builtin_tools(&mut self) {
        self.register(Tool {
            name: "swap_tokens".to_string(),
            description: "Swap tokens on DEX".to_string(),
            capability_hash: String::new(), // computed below
            wasm_module: None,
        });

        self.register(Tool {
            name: "transfer".to_string(),
            description: "Transfer tokens".to_string(),
            capability_hash: String::new(),
            wasm_module: None,
        });

        self.register(Tool {
            name: "query".to_string(),
            description: "Query blockchain state".to_string(),
            capability_hash: String::new(),
            wasm_module: None,
        });
    }

    pub fn register(&mut self, mut tool: Tool) {
        // Always (re)compute capability hash from tool definition
        tool.capability_hash = compute_capability_hash(
            &tool.name,
            &tool.description,
            &tool.wasm_module,
        );
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    pub fn list(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_hash_is_deterministic() {
        let hash1 = compute_capability_hash("swap_tokens", "Swap tokens on DEX", &None);
        let hash2 = compute_capability_hash("swap_tokens", "Swap tokens on DEX", &None);
        assert_eq!(hash1, hash2);
        assert!(hash1.starts_with("0x"));
        assert_eq!(hash1.len(), 66); // 0x + 64 hex chars
    }

    #[test]
    fn test_different_tools_have_different_hashes() {
        let registry = ToolRegistry::new();
        let swap = registry.get("swap_tokens").unwrap();
        let transfer = registry.get("transfer").unwrap();
        assert_ne!(swap.capability_hash, transfer.capability_hash);
    }

    #[test]
    fn test_wasm_module_changes_hash() {
        let hash_no_wasm = compute_capability_hash("test", "test tool", &None);
        let hash_with_wasm = compute_capability_hash("test", "test tool", &Some(vec![0, 1, 2]));
        assert_ne!(hash_no_wasm, hash_with_wasm);
    }
}
