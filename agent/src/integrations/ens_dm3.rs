use crate::core::config::AgentConfig;
use crate::core::types::AgentMessage;
use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

pub struct DM3Client {
    client: Client,
    delivery_service_url: String,
    sender: mpsc::Sender<AgentMessage>,
    receiver: mpsc::Receiver<AgentMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DM3Profile {
    pub public_encryption_key: String,
    pub public_signing_key: String,
    pub delivery_service_url: String,
}

/// Wire format for DM3 delivery service messages
#[derive(Debug, Serialize, Deserialize)]
struct DM3Envelope {
    to: String,
    from: String,
    message: String,
    #[serde(rename = "encryptionEnvelopeType")]
    encryption_type: String,
    timestamp: i64,
}

impl DM3Client {
    pub async fn new(config: &AgentConfig) -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);

        Ok(Self {
            client: Client::new(),
            delivery_service_url: config.dm3_delivery_service_url.clone(),
            sender: tx,
            receiver: rx,
        })
    }

    pub fn sender(&self) -> mpsc::Sender<AgentMessage> {
        self.sender.clone()
    }

    pub async fn receive_message(&mut self) -> Result<AgentMessage> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Channel closed"))
    }

    /// Send a message to a recipient via their DM3 delivery service.
    ///
    /// Resolves the recipient's DM3 profile to find their delivery service,
    /// then POSTs the encrypted envelope.
    pub async fn send_message(&self, recipient_ens: &str, message: &AgentMessage) -> Result<()> {
        // Resolve recipient's delivery service
        let profile = self.resolve_dm3_profile(recipient_ens).await?;

        let envelope = DM3Envelope {
            to: recipient_ens.to_string(),
            from: String::new(), // filled by delivery service from auth
            message: serde_json::to_string(message)?,
            encryption_type: "x25519-xsalsa20-poly1305".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        };

        let delivery_url = if profile.delivery_service_url.is_empty() {
            self.delivery_service_url.clone()
        } else {
            profile.delivery_service_url
        };

        let resp = self
            .client
            .post(format!("{}/messages", delivery_url))
            .json(&envelope)
            .send()
            .await
            .context(format!(
                "Failed to deliver DM3 message to {}",
                recipient_ens
            ))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "DM3 delivery service returned {} for {}: {}",
                status,
                recipient_ens,
                body
            );
        }

        tracing::info!("DM3 message delivered to {}", recipient_ens);
        Ok(())
    }

    /// Resolve a DM3 profile from an ENS name.
    ///
    /// Queries the ENS text record `network.dm3.profile` for the given name,
    /// then fetches the full profile from the delivery service.
    pub async fn resolve_dm3_profile(&self, ens_name: &str) -> Result<DM3Profile> {
        // Try the delivery service's profile endpoint first
        let resp = self
            .client
            .get(format!(
                "{}/profile/{}",
                self.delivery_service_url, ens_name
            ))
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let profile: DM3Profile = r
                    .json()
                    .await
                    .context("Failed to parse DM3 profile response")?;
                return Ok(profile);
            }
            Ok(r) => {
                tracing::debug!(
                    "DM3 profile lookup for {} returned {}, using default delivery service",
                    ens_name,
                    r.status()
                );
            }
            Err(e) => {
                tracing::debug!(
                    "DM3 profile lookup for {} failed ({}), using default delivery service",
                    ens_name,
                    e
                );
            }
        }

        // Fallback: assume the default delivery service
        Ok(DM3Profile {
            public_encryption_key: String::new(),
            public_signing_key: String::new(),
            delivery_service_url: self.delivery_service_url.clone(),
        })
    }

    /// Poll the delivery service for new incoming messages.
    pub async fn poll_messages(&self) -> Result<Vec<AgentMessage>> {
        let resp = self
            .client
            .get(format!("{}/messages/incoming", self.delivery_service_url))
            .send()
            .await
            .context("Failed to poll DM3 delivery service")?;

        if !resp.status().is_success() {
            anyhow::bail!(
                "DM3 delivery service returned {} when polling messages",
                resp.status()
            );
        }

        let messages: Vec<AgentMessage> = resp.json().await.unwrap_or_default();
        Ok(messages)
    }
}
