use crate::core::config::AgentConfig;
use crate::core::types::AgentMessage;
use anyhow::{Result, Context};
use ethers::utils::keccak256;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// ENS Registry address (same on mainnet and Sepolia).
const ENS_REGISTRY: &str = "0x00000000000C2e074eC69A0dFb2997BA6C7d2e1e";

pub struct DM3Client {
    client: Client,
    delivery_service_url: String,
    rpc_url: String,
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
            rpc_url: config.rpc_url.clone(),
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
    /// First attempts on-chain resolution by reading the `network.dm3.profile`
    /// ENS text record via JSON-RPC `eth_call`. Falls back to querying the
    /// delivery service HTTP endpoint, then to a bare default.
    pub async fn resolve_dm3_profile(&self, ens_name: &str) -> Result<DM3Profile> {
        // ── 1. On-chain ENS text-record resolution ──
        match self.resolve_dm3_profile_from_ens(ens_name).await {
            Ok(Some(profile)) => {
                tracing::info!(
                    "Resolved DM3 profile for {} from ENS text record",
                    ens_name
                );
                return Ok(profile);
            }
            Ok(None) => {
                tracing::debug!(
                    "No network.dm3.profile ENS text record for {}",
                    ens_name
                );
            }
            Err(e) => {
                tracing::debug!(
                    "ENS text record resolution failed for {}: {}",
                    ens_name,
                    e
                );
            }
        }

        // ── 2. HTTP fallback via delivery service ──
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

        // ── 3. Bare fallback ──
        Ok(DM3Profile {
            public_encryption_key: String::new(),
            public_signing_key: String::new(),
            delivery_service_url: self.delivery_service_url.clone(),
        })
    }

    // ── ENS on-chain helpers ─────────────────────────────────────────────

    /// Resolve the `network.dm3.profile` text record from ENS for `ens_name`.
    /// Returns `Ok(None)` when the record simply doesn't exist.
    async fn resolve_dm3_profile_from_ens(
        &self,
        ens_name: &str,
    ) -> Result<Option<DM3Profile>> {
        let node = namehash(ens_name);
        let node_hex = hex::encode(node);

        // Step 1 — ask the ENS Registry for the resolver address.
        // selector: resolver(bytes32) = 0x0178b8bf
        let calldata = format!("0x0178b8bf{}", node_hex);
        let resolver_result = self.eth_call(ENS_REGISTRY, &calldata).await?;

        let resolver_addr = parse_address_from_result(&resolver_result)
            .context("Failed to parse resolver address from ENS registry")?;

        if resolver_addr == "0x0000000000000000000000000000000000000000" {
            return Ok(None);
        }

        // Step 2 — call text(bytes32 node, string key) on the resolver.
        // selector: text(bytes32,string) = 0x59d1d43c
        let key = "network.dm3.profile";
        let key_bytes = key.as_bytes();

        // ABI encode: node (32 bytes) + offset to string (0x40) + string length + string data (padded)
        let offset = "0000000000000000000000000000000000000000000000000000000000000040";
        let str_len = format!("{:064x}", key_bytes.len());
        let key_hex = hex::encode(key_bytes);
        let padded_key = right_pad_hex(&key_hex, 64);

        let text_calldata = format!(
            "0x59d1d43c{}{}{}{}",
            node_hex, offset, str_len, padded_key,
        );
        let text_result = self.eth_call(&resolver_addr, &text_calldata).await?;

        // Decode ABI-encoded string response.
        let profile_json = decode_abi_string(&text_result);
        let profile_json = match profile_json {
            Some(s) if !s.is_empty() => s,
            _ => return Ok(None),
        };

        // Step 3 — parse the DM3 profile JSON.
        let profile: DM3Profile = serde_json::from_str(&profile_json)
            .context("Failed to parse DM3 profile JSON from ENS text record")?;
        Ok(Some(profile))
    }

    /// Low-level `eth_call` via JSON-RPC.
    async fn eth_call(&self, to: &str, data: &str) -> Result<String> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": [{ "to": to, "data": data }, "latest"],
            "id": 1
        });

        let resp = self
            .client
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .context("eth_call request failed")?;

        let json: serde_json::Value = resp.json().await.context("eth_call response not JSON")?;

        if let Some(err) = json.get("error") {
            anyhow::bail!("eth_call RPC error: {}", err);
        }

        json.get("result")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("eth_call returned no result"))
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

// ── Free functions: ENS helpers ──────────────────────────────────────────────

/// Compute the ENS namehash of `name`.
///
/// namehash("") = 0x00..00
/// namehash("eth") = keccak256(namehash("") + keccak256("eth"))
fn namehash(name: &str) -> [u8; 32] {
    let mut node = [0u8; 32];
    if name.is_empty() {
        return node;
    }
    for label in name.rsplit('.') {
        let label_hash = keccak256(label.as_bytes());
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&node);
        combined[32..].copy_from_slice(&label_hash);
        node = keccak256(&combined);
    }
    node
}

/// Extract a 20-byte address from a 32-byte ABI-encoded hex result.
fn parse_address_from_result(hex_result: &str) -> Result<String> {
    let clean = hex_result.strip_prefix("0x").unwrap_or(hex_result);
    if clean.len() < 40 {
        anyhow::bail!("result too short to contain an address");
    }
    // The address sits in the last 40 hex chars of the 64-char word.
    let addr_hex = &clean[clean.len().saturating_sub(40)..];
    Ok(format!("0x{}", addr_hex))
}

/// Decode an ABI-encoded `string` return value.
///
/// Layout: offset (32 bytes) | length (32 bytes) | data (padded to 32-byte boundary)
fn decode_abi_string(hex_result: &str) -> Option<String> {
    let clean = hex_result.strip_prefix("0x").unwrap_or(hex_result);
    if clean.len() < 128 {
        return None;
    }
    // First 64 hex chars = offset (in bytes) into the data area.
    let offset = usize::from_str_radix(&clean[..64], 16).ok()? * 2; // convert byte offset to hex-char offset
    if offset + 64 > clean.len() {
        return None;
    }
    let str_len = usize::from_str_radix(&clean[offset..offset + 64], 16).ok()?;
    if str_len == 0 || str_len > 100_000 {
        return None;
    }
    let data_start = offset + 64;
    let data_end = data_start + str_len * 2;
    if data_end > clean.len() {
        return None;
    }
    let bytes = hex::decode(&clean[data_start..data_end]).ok()?;
    String::from_utf8(bytes).ok()
}

/// Right-pad a hex string with zeros to a multiple of `chunk` hex characters.
fn right_pad_hex(hex_str: &str, chunk: usize) -> String {
    let remainder = hex_str.len() % chunk;
    if remainder == 0 {
        hex_str.to_string()
    } else {
        let pad = chunk - remainder;
        format!("{}{}", hex_str, "0".repeat(pad))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namehash_empty() {
        assert_eq!(namehash(""), [0u8; 32]);
    }

    #[test]
    fn test_namehash_eth() {
        // Well-known: namehash("eth") = 0x93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae
        let expected =
            hex::decode("93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae")
                .unwrap();
        assert_eq!(namehash("eth"), expected.as_slice());
    }

    #[test]
    fn test_namehash_vitalik_eth() {
        // namehash("vitalik.eth") = 0xee6c4522aab0003e8d14cd40a6af439055fd2577951148c14b6cea9a53475835
        let expected =
            hex::decode("ee6c4522aab0003e8d14cd40a6af439055fd2577951148c14b6cea9a53475835")
                .unwrap();
        assert_eq!(namehash("vitalik.eth"), expected.as_slice());
    }

    #[test]
    fn test_decode_abi_string_empty() {
        assert_eq!(decode_abi_string("0x"), None);
    }

    #[test]
    fn test_right_pad_hex() {
        assert_eq!(right_pad_hex("abcd", 64), format!("abcd{}", "0".repeat(60)));
        assert_eq!(
            right_pad_hex(&"0".repeat(64), 64),
            "0".repeat(64)
        );
    }
}
