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

    // Compute policy hash from the actual policy config
    let policy_str = format!(
        "{:?}{:?}{}",
        s.config.policy.allowed_tools,
        s.config.policy.endpoint_allowlist,
        s.config.policy.max_value_autonomous_wei
    );
    use sha2::Digest as _;
    let hash_bytes = sha2::Sha256::digest(policy_str.as_bytes());
    let policy_hash = format!("0x{}", hex::encode(hash_bytes));

    Json(StatusResponse {
        agent_id: s.config.agent_id.clone(),
        ens_name: s.config.ens_name.clone(),
        status: "online".to_string(),
        uptime_secs: uptime,
        network: if s.config.rpc_url.contains("sepolia") {
            "sepolia".to_string()
        } else if s.config.rpc_url.contains("testnet") {
            "testnet".to_string()
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

    s.stats.total_actions += 1;

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

// ========== STATE MUTATION (called by agent runtime) ==========

impl AgentState {
    /// Record a completed proof in the agent state.
    pub fn record_proof(&mut self, proof: ProofRecord) {
        self.stats.proofs_generated += 1;
        self.stats.total_actions += 1;

        if proof.status == "verified" {
            self.stats.proofs_verified += 1;
        }

        match proof.approval_type.as_str() {
            "Autonomous" => self.stats.autonomous_actions += 1,
            "Ledger Approved" => self.stats.approved_actions += 1,
            _ => {}
        }

        self.activity.insert(0, ActivityItem {
            activity_type: "proof".to_string(),
            title: "Proof Generated".to_string(),
            description: format!(
                "{} {} | {} | {}",
                proof.action, proof.value, proof.approval_type, proof.status
            ),
            timestamp: proof.timestamp,
        });

        self.proofs.insert(0, proof);
    }

    /// Record a policy violation.
    pub fn record_violation(&mut self, rule: &str, details: &str) {
        self.stats.violations += 1;
        self.activity.insert(0, ActivityItem {
            activity_type: "violation".to_string(),
            title: format!("Policy Violation: {}", rule),
            description: details.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        });
    }

    /// Record an incoming DM3 message.
    pub fn record_incoming_message(&mut self, msg: MessageRecord) {
        self.activity.insert(0, ActivityItem {
            activity_type: "message".to_string(),
            title: "DM3 Message Received".to_string(),
            description: format!("From {} via encrypted channel", msg.from),
            timestamp: msg.timestamp,
        });
        self.messages.push(msg);
    }
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

/// Create initial agent state — starts empty, populated by real agent activity.
pub fn create_initial_state(config: AgentConfig) -> SharedState {
    let now = chrono::Utc::now().timestamp();

    Arc::new(RwLock::new(AgentState {
        config,
        stats: AgentStats {
            total_actions: 0,
            autonomous_actions: 0,
            approved_actions: 0,
            violations: 0,
            proofs_generated: 0,
            proofs_verified: 0,
        },
        activity: Vec::new(),
        proofs: Vec::new(),
        messages: Vec::new(),
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
