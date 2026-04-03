use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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
            capability_hash: "0x1234".to_string(),
            wasm_module: None,
        });
        
        self.register(Tool {
            name: "transfer".to_string(),
            description: "Transfer tokens".to_string(),
            capability_hash: "0x5678".to_string(),
            wasm_module: None,
        });
        
        self.register(Tool {
            name: "query".to_string(),
            description: "Query blockchain state".to_string(),
            capability_hash: "0x9abc".to_string(),
            wasm_module: None,
        });
    }
    
    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }
    
    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }
    
    pub fn list(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }
}
