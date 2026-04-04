use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::core::config::AgentConfig;
use crate::core::types::*;

/// Shared agent state accessible by API handlers
#[derive(Debug, Clone)]
pub struct AgentState {
    pub config: AgentConfig,
    pub stats: AgentStats,
    pub activity: Vec<ActivityItem>,
    pub proofs: Vec<ProofRecord>,
    pub messages: Vec<MessageRecord>,
    pub started_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStats {
    pub total_actions: u64,
    pub autonomous_actions: u64,
    pub approved_actions: u64,
    pub violations: u64,
    pub proofs_generated: u64,
    pub proofs_verified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub activity_type: String, // "proof", "message", "approval"
    pub title: String,
    pub description: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofRecord {
    pub proof_id: String,
    pub agent_id: String,
    pub action: String,
    pub value: String,
    pub approval_type: String,
    pub status: String, // "verified", "pending", "failed"
    pub timestamp: i64,
    pub proof_time_secs: u64,
    pub output_commitment: String,
    pub tx_hash: Option<String>,
    pub block_number: Option<u64>,
    pub policy_checks: Vec<PolicyCheckRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheckRecord {
    pub rule: String,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: i64,
    pub encrypted: bool,
    pub delivered: bool,
}

pub type SharedState = Arc<RwLock<AgentState>>;

// ========== API RESPONSE TYPES ==========

#[derive(Serialize)]
pub struct StatusResponse {
    pub agent_id: String,
    pub ens_name: String,
    pub status: String,
    pub uptime_secs: i64,
    pub network: String,
    pub policy_hash: String,
    pub allowed_tools: Vec<String>,
    pub endpoint_allowlist: Vec<String>,
    pub max_value_autonomous_wei: u64,
    pub stats: AgentStats,
}

#[derive(Serialize)]
pub struct ActivityResponse {
    pub items: Vec<ActivityItem>,
}

#[derive(Serialize)]
pub struct ProofsResponse {
    pub proofs: Vec<ProofRecord>,
}

#[derive(Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<MessageRecord>,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub to: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct SendMessageResponse {
    pub success: bool,
    pub message_id: String,
}

// ========== HANDLERS ==========

async fn get_status(State(state): State<SharedState>) -> Json<StatusResponse> {
    let s = state.read().await;
    let now = chrono::Utc::now().timestamp();
    let uptime = now - s.started_at;

    // Compute a mock policy hash from config
    let policy_str = format!(
        "{:?}{:?}{}",
        s.config.policy.allowed_tools,
        s.config.policy.endpoint_allowlist,
        s.config.policy.max_value_autonomous_wei
    );
    use sha2::Digest as _;
    let hash_bytes = sha2::Sha256::digest(policy_str.as_bytes());
    let policy_hash = format!("0x{}", &hex::encode(hash_bytes)[..16]);

    Json(StatusResponse {
        agent_id: s.config.agent_id.clone(),
        ens_name: s.config.ens_name.clone(),
        status: "online".to_string(),
        uptime_secs: uptime,
        network: if s.config.rpc_url.contains("sepolia") {
            "sepolia".to_string()
        } else {
            "mainnet".to_string()
        },
        policy_hash,
        allowed_tools: s.config.policy.allowed_tools.clone(),
        endpoint_allowlist: s.config.policy.endpoint_allowlist.clone(),
        max_value_autonomous_wei: s.config.policy.max_value_autonomous_wei,
        stats: s.stats.clone(),
    })
}

async fn get_activity(State(state): State<SharedState>) -> Json<ActivityResponse> {
    let s = state.read().await;
    Json(ActivityResponse {
        items: s.activity.clone(),
    })
}

async fn get_proofs(State(state): State<SharedState>) -> Json<ProofsResponse> {
    let s = state.read().await;
    Json(ProofsResponse {
        proofs: s.proofs.clone(),
    })
}

async fn get_messages(State(state): State<SharedState>) -> Json<MessagesResponse> {
    let s = state.read().await;
    Json(MessagesResponse {
        messages: s.messages.clone(),
    })
}

async fn send_message(
    State(state): State<SharedState>,
    Json(req): Json<SendMessageRequest>,
) -> (StatusCode, Json<SendMessageResponse>) {
    let mut s = state.write().await;
    let msg = MessageRecord {
        from: s.config.agent_id.clone(),
        to: req.to.clone(),
        content: req.content.clone(),
        timestamp: chrono::Utc::now().timestamp(),
        encrypted: true,
        delivered: true,
    };
    s.messages.push(msg);

    s.activity.insert(0, ActivityItem {
        activity_type: "message".to_string(),
        title: "DM3 Message Sent".to_string(),
        description: format!("To {} via encrypted DM3", req.to),
        timestamp: chrono::Utc::now().timestamp(),
    });

    (StatusCode::OK, Json(SendMessageResponse {
        success: true,
        message_id: uuid::Uuid::new_v4().to_string(),
    }))
}

async fn health() -> &'static str {
    "ok"
}

// ========== SERVER SETUP ==========

pub fn create_router(state: SharedState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/api/status", get(get_status))
        .route("/api/activity", get(get_activity))
        .route("/api/proofs", get(get_proofs))
        .route("/api/messages", get(get_messages))
        .route("/api/messages/send", post(send_message))
        .with_state(state)
        .layer(cors)
}

pub fn create_initial_state(config: AgentConfig) -> SharedState {
    let now = chrono::Utc::now().timestamp();

    // Seed with some demo data
    let activity = vec![
        ActivityItem {
            activity_type: "proof".to_string(),
            title: "Proof Generated".to_string(),
            description: format!("Swap 100 USDC -> ETH | Autonomous | Verified on-chain"),
            timestamp: now - 120,
        },
        ActivityItem {
            activity_type: "message".to_string(),
            title: "DM3 Message Received".to_string(),
            description: "From bob.proofclaw.eth via encrypted channel".to_string(),
            timestamp: now - 300,
        },
        ActivityItem {
            activity_type: "approval".to_string(),
            title: "Ledger Approval Granted".to_string(),
            description: "Transfer 500 USDC | Owner signed via Ledger".to_string(),
            timestamp: now - 600,
        },
    ];

    let proofs = vec![
        ProofRecord {
            proof_id: "POC-1234".to_string(),
            agent_id: config.agent_id.clone(),
            action: "swap_tokens".to_string(),
            value: "100 USDC".to_string(),
            approval_type: "Autonomous".to_string(),
            status: "verified".to_string(),
            timestamp: now - 120,
            proof_time_secs: 42,
            output_commitment: "0xabcd1234ef567890abcd1234ef567890abcd1234ef567890abcd1234ef567890".to_string(),
            tx_hash: Some("0x9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba".to_string()),
            block_number: Some(12345678),
            policy_checks: vec![
                PolicyCheckRecord { rule: "Tool Allowlist".to_string(), passed: true, details: "swap_tokens is in allowed tools list".to_string() },
                PolicyCheckRecord { rule: "Value Threshold".to_string(), passed: true, details: "100 USDC <= autonomous limit".to_string() },
                PolicyCheckRecord { rule: "Endpoint Allowlist".to_string(), passed: true, details: "api.uniswap.org is approved".to_string() },
                PolicyCheckRecord { rule: "Prompt Injection".to_string(), passed: true, details: "No injection patterns detected".to_string() },
            ],
        },
        ProofRecord {
            proof_id: "POC-1235".to_string(),
            agent_id: config.agent_id.clone(),
            action: "transfer".to_string(),
            value: "500 USDC".to_string(),
            approval_type: "Ledger Approved".to_string(),
            status: "verified".to_string(),
            timestamp: now - 600,
            proof_time_secs: 38,
            output_commitment: "0xef012345ab678901ef012345ab678901ef012345ab678901ef012345ab678901".to_string(),
            tx_hash: Some("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()),
            block_number: Some(12345650),
            policy_checks: vec![
                PolicyCheckRecord { rule: "Tool Allowlist".to_string(), passed: true, details: "transfer is in allowed tools list".to_string() },
                PolicyCheckRecord { rule: "Value Threshold".to_string(), passed: true, details: "500 USDC > limit -> Ledger approval required".to_string() },
                PolicyCheckRecord { rule: "Ledger Signature".to_string(), passed: true, details: "Valid signature from owner's Ledger device".to_string() },
                PolicyCheckRecord { rule: "Clear Signing".to_string(), passed: true, details: "ERC-7730 metadata displayed on Ledger screen".to_string() },
            ],
        },
        ProofRecord {
            proof_id: "POC-1236".to_string(),
            agent_id: config.agent_id.clone(),
            action: "query".to_string(),
            value: "0 USDC".to_string(),
            approval_type: "Autonomous".to_string(),
            status: "pending".to_string(),
            timestamp: now,
            proof_time_secs: 0,
            output_commitment: "0x5678abcd9012ef345678abcd9012ef345678abcd9012ef345678abcd9012ef34".to_string(),
            tx_hash: None,
            block_number: None,
            policy_checks: vec![],
        },
    ];

    let messages = vec![
        MessageRecord {
            from: "bob.proofclaw.eth".to_string(),
            to: config.ens_name.clone(),
            content: "Hey! I'm looking to swap 500 USDC for ETH. Are you interested?".to_string(),
            timestamp: now - 720,
            encrypted: true,
            delivered: true,
        },
        MessageRecord {
            from: config.ens_name.clone(),
            to: "bob.proofclaw.eth".to_string(),
            content: "Sure! Let me check my policy limits...".to_string(),
            timestamp: now - 600,
            encrypted: true,
            delivered: true,
        },
        MessageRecord {
            from: config.ens_name.clone(),
            to: "bob.proofclaw.eth".to_string(),
            content: "500 USDC is above my autonomous threshold. I'll need Ledger approval. Want to proceed?".to_string(),
            timestamp: now - 540,
            encrypted: true,
            delivered: true,
        },
        MessageRecord {
            from: "bob.proofclaw.eth".to_string(),
            to: config.ens_name.clone(),
            content: "Same here - let's both submit for approval and execute when ready.".to_string(),
            timestamp: now - 420,
            encrypted: true,
            delivered: true,
        },
    ];

    Arc::new(RwLock::new(AgentState {
        config,
        stats: AgentStats {
            total_actions: 47,
            autonomous_actions: 42,
            approved_actions: 4,
            violations: 1,
            proofs_generated: 45,
            proofs_verified: 42,
        },
        activity,
        proofs,
        messages,
        started_at: now,
    }))
}

pub async fn start_api_server(state: SharedState, port: u16) -> anyhow::Result<()> {
    let app = create_router(state);
    let addr = format!("0.0.0.0:{}", port);
    info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
