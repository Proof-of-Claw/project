use super::config::AgentConfig;
use super::types::*;
use super::intent_router::IntentRouter;
use super::job_scheduler::JobScheduler;
use crate::tools::registry::ToolRegistry;
use crate::safety::policy_engine::PolicyEngine;
use crate::integrations::zero_g::{ZeroGCompute, ZeroGStorage};
use crate::integrations::ens_dm3::DM3Client;
use anyhow::Result;
use tracing::{info, warn, error};
use uuid::Uuid;

pub struct Agent {
    config: AgentConfig,
    tool_registry: ToolRegistry,
    policy_engine: PolicyEngine,
    zero_g_compute: ZeroGCompute,
    zero_g_storage: ZeroGStorage,
    dm3_client: DM3Client,
    intent_router: IntentRouter,
    job_scheduler: JobScheduler,
}

impl Agent {
    pub async fn new(config: AgentConfig) -> Result<Self> {
        info!("Initializing agent: {}", config.agent_id);
        
        let tool_registry = ToolRegistry::new();
        let policy_engine = PolicyEngine::new(config.policy.clone());
        let zero_g_compute = ZeroGCompute::new(&config).await?;
        let zero_g_storage = ZeroGStorage::new(&config).await?;
        let dm3_client = DM3Client::new(&config).await?;
        let intent_router = IntentRouter::new();
        let job_scheduler = JobScheduler::new();
        
        Ok(Self {
            config,
            tool_registry,
            policy_engine,
            zero_g_compute,
            zero_g_storage,
            dm3_client,
            intent_router,
            job_scheduler,
        })
    }
    
    pub fn id(&self) -> &str {
        &self.config.agent_id
    }
    
    pub async fn run(&mut self) -> Result<()> {
        info!("Agent {} starting main loop", self.config.agent_id);
        
        loop {
            tokio::select! {
                message = self.dm3_client.receive_message() => {
                    if let Ok(msg) = message {
                        self.handle_message(msg).await?;
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutting down agent");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_message(&self, message: AgentMessage) -> Result<()> {
        info!("Received message type: {:?}", message.message_type);
        
        let session_id = Uuid::new_v4().to_string();
        let mut trace = ExecutionTrace {
            agent_id: self.config.agent_id.clone(),
            session_id: session_id.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            inference_commitment: String::new(),
            tool_invocations: Vec::new(),
            policy_check_results: Vec::new(),
            output_commitment: String::new(),
        };
        
        let intent = self.intent_router.classify_intent(&message)?;
        
        let inference_request = InferenceRequest {
            system_prompt: format!("You are {}, an autonomous agent with policy constraints.", self.config.agent_id),
            user_prompt: format!("Action: {}, Params: {:?}", message.payload.action, message.payload.params),
            model: None,
        };
        
        let inference_response = self.zero_g_compute.inference(&inference_request).await?;
        trace.inference_commitment = inference_response.attestation_signature.clone();
        
        let policy_check = self.policy_engine.check(&message, &inference_response)?;
        trace.policy_check_results.push(policy_check.clone());
        
        if matches!(policy_check.severity, PolicySeverity::Block) {
            warn!("Action blocked by policy: {}", policy_check.details);
            return Ok(());
        }
        
        let trace_hash = self.zero_g_storage.store_trace(&trace).await?;
        info!("Execution trace stored: {}", trace_hash);
        
        Ok(())
    }
}
