#[cfg(test)]
mod tests {
    use super::super::registry::{Tool, ToolRegistry};

    #[test]
    fn test_registry_initialization() {
        let registry = ToolRegistry::new();
        let tools = registry.list();
        
        assert!(tools.len() >= 3);
    }

    #[test]
    fn test_get_existing_tool() {
        let registry = ToolRegistry::new();
        let tool = registry.get("swap_tokens");
        
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "swap_tokens");
    }

    #[test]
    fn test_get_nonexistent_tool() {
        let registry = ToolRegistry::new();
        let tool = registry.get("nonexistent");
        
        assert!(tool.is_none());
    }

    #[test]
    fn test_register_custom_tool() {
        let mut registry = ToolRegistry::new();
        let custom_tool = Tool {
            name: "custom_action".to_string(),
            description: "Custom tool".to_string(),
            capability_hash: "0xabcd".to_string(),
            wasm_module: None,
        };
        
        registry.register(custom_tool);
        let retrieved = registry.get("custom_action");
        
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().description, "Custom tool");
    }

    #[test]
    fn test_builtin_tools_present() {
        let registry = ToolRegistry::new();
        
        assert!(registry.get("swap_tokens").is_some());
        assert!(registry.get("transfer").is_some());
        assert!(registry.get("query").is_some());
    }
}
