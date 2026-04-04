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
        env::set_var("PRIVATE_KEY", "0xdeadbeef1234567890abcdef");
        env::set_var("RPC_URL", "https://eth-sepolia.example.com/v2/key123");
        env::set_var("ZERO_G_INDEXER_RPC", "https://indexer-storage-testnet.0g.ai");
        env::set_var("ZERO_G_COMPUTE_ENDPOINT", "https://broker-testnet.0g.ai");
        env::set_var("DM3_DELIVERY_SERVICE_URL", "https://dm3.example.com");
        env::set_var("ALLOWED_TOOLS", "swap,transfer");
        env::set_var("ENDPOINT_ALLOWLIST", "https://api.uniswap.org");
        env::set_var("MAX_VALUE_AUTONOMOUS_WEI", "1000000000000000000");

        let config = AgentConfig::from_env();

        env::remove_var("AGENT_ID");
        env::remove_var("ENS_NAME");
        env::remove_var("PRIVATE_KEY");
        env::remove_var("RPC_URL");
        env::remove_var("ZERO_G_INDEXER_RPC");
        env::remove_var("ZERO_G_COMPUTE_ENDPOINT");
        env::remove_var("DM3_DELIVERY_SERVICE_URL");
        env::remove_var("ALLOWED_TOOLS");
        env::remove_var("ENDPOINT_ALLOWLIST");
        env::remove_var("MAX_VALUE_AUTONOMOUS_WEI");

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.agent_id, "test-agent");
        assert_eq!(config.ens_name, "test.eth");
    }

    #[test]
    fn test_agent_config_rejects_zero_private_key() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();

        env::set_var("AGENT_ID", "test-agent");
        env::set_var("ENS_NAME", "test.eth");
        env::set_var("PRIVATE_KEY", "0x0000000000000000000000000000000000000000000000000000000000000000");
        env::set_var("RPC_URL", "https://eth-sepolia.example.com/v2/key123");
        env::set_var("ZERO_G_INDEXER_RPC", "https://indexer-storage-testnet.0g.ai");
        env::set_var("ZERO_G_COMPUTE_ENDPOINT", "https://broker-testnet.0g.ai");
        env::set_var("DM3_DELIVERY_SERVICE_URL", "https://dm3.example.com");
        env::set_var("ALLOWED_TOOLS", "swap");
        env::set_var("ENDPOINT_ALLOWLIST", "https://api.uniswap.org");
        env::set_var("MAX_VALUE_AUTONOMOUS_WEI", "1000000000000000000");

        let config = AgentConfig::from_env();

        env::remove_var("AGENT_ID");
        env::remove_var("ENS_NAME");
        env::remove_var("PRIVATE_KEY");
        env::remove_var("RPC_URL");
        env::remove_var("ZERO_G_INDEXER_RPC");
        env::remove_var("ZERO_G_COMPUTE_ENDPOINT");
        env::remove_var("DM3_DELIVERY_SERVICE_URL");
        env::remove_var("ALLOWED_TOOLS");
        env::remove_var("ENDPOINT_ALLOWLIST");
        env::remove_var("MAX_VALUE_AUTONOMOUS_WEI");

        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("all zeros"));
    }

    #[test]
    fn test_agent_config_missing_required() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();

        env::remove_var("AGENT_ID");
        env::remove_var("ENS_NAME");
        env::remove_var("PRIVATE_KEY");
        env::remove_var("RPC_URL");

        let config = AgentConfig::from_env();
        assert!(config.is_err());
    }

    #[test]
    fn test_zero_addresses_become_none() {
        use std::sync::Mutex;
        static ENV_LOCK: Mutex<()> = Mutex::new(());
        let _guard = ENV_LOCK.lock().unwrap();

        env::set_var("AGENT_ID", "test-agent");
        env::set_var("ENS_NAME", "test.eth");
        env::set_var("PRIVATE_KEY", "0xdeadbeef1234567890abcdef");
        env::set_var("RPC_URL", "https://eth-sepolia.example.com/v2/key123");
        env::set_var("ZERO_G_INDEXER_RPC", "https://indexer-storage-testnet.0g.ai");
        env::set_var("ZERO_G_COMPUTE_ENDPOINT", "https://broker-testnet.0g.ai");
        env::set_var("DM3_DELIVERY_SERVICE_URL", "https://dm3.example.com");
        env::set_var("ALLOWED_TOOLS", "swap");
        env::set_var("ENDPOINT_ALLOWLIST", "https://api.uniswap.org");
        env::set_var("MAX_VALUE_AUTONOMOUS_WEI", "1000000000000000000");
        env::set_var("EIP8004_IDENTITY_REGISTRY", "0x0000000000000000000000000000000000000000");
        env::set_var("INFT_CONTRACT", "0x0000000000000000000000000000000000000000");
        env::set_var("RISC_ZERO_IMAGE_ID", "0x0000000000000000000000000000000000000000000000000000000000000000");

        let config = AgentConfig::from_env().unwrap();

        env::remove_var("AGENT_ID");
        env::remove_var("ENS_NAME");
        env::remove_var("PRIVATE_KEY");
        env::remove_var("RPC_URL");
        env::remove_var("ZERO_G_INDEXER_RPC");
        env::remove_var("ZERO_G_COMPUTE_ENDPOINT");
        env::remove_var("DM3_DELIVERY_SERVICE_URL");
        env::remove_var("ALLOWED_TOOLS");
        env::remove_var("ENDPOINT_ALLOWLIST");
        env::remove_var("MAX_VALUE_AUTONOMOUS_WEI");
        env::remove_var("EIP8004_IDENTITY_REGISTRY");
        env::remove_var("INFT_CONTRACT");
        env::remove_var("RISC_ZERO_IMAGE_ID");

        // All-zero addresses should be treated as None (not configured)
        assert!(config.eip8004_identity_registry.is_none());
        assert!(config.inft_contract.is_none());
        assert!(config.risc_zero_image_id.is_none());
    }
}
