#[cfg(test)]
mod tests {
    use super::super::config::{AgentConfig, PolicyConfig};
    use std::env;

    #[test]
    fn test_policy_config_defaults() {
        let policy = PolicyConfig {
            allowed_tools: vec!["swap".to_string(), "transfer".to_string()],
            endpoint_allowlist: vec!["https://api.example.com".to_string()],
            max_value_autonomous_wei: 1_000_000_000_000_000_000,
        };

        assert_eq!(policy.allowed_tools.len(), 2);
        assert!(policy.allowed_tools.contains(&"swap".to_string()));
        assert_eq!(policy.max_value_autonomous_wei, 1_000_000_000_000_000_000);
    }

    #[test]
    fn test_agent_config_from_env() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();
        
        env::set_var("AGENT_ID", "test-agent");
        env::set_var("ENS_NAME", "test.eth");
        env::set_var("PRIVATE_KEY", "0x1234");

        let config = AgentConfig::from_env();
        
        env::remove_var("AGENT_ID");
        env::remove_var("ENS_NAME");
        env::remove_var("PRIVATE_KEY");
        
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.agent_id, "test-agent");
        assert_eq!(config.ens_name, "test.eth");
        assert_eq!(config.private_key, "0x1234");
    }

    #[test]
    fn test_agent_config_missing_required() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();
        
        env::remove_var("AGENT_ID");
        env::remove_var("ENS_NAME");
        env::remove_var("PRIVATE_KEY");

        let config = AgentConfig::from_env();
        assert!(config.is_err());
    }
}
