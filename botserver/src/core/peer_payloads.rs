//! Opt-in peer-network payloads (#1866).
//!
//! Defines the durable payload kinds a peer may exchange over the mesh:
//! personal mesh, shared folder, drop box, ask-my-bot, peer backup, calls and
//! peer cache. Payloads are end-to-end encrypted and the relay retains only
//! an encrypted copy that expires; nodes never retain user content in
//! plaintext.
//!
//! This is the data layer only. Transport wiring lives in the peer-network
//! foundation (`peer_network.rs`); HTTP surface is added by the routes
//! module.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use uuid::Uuid;

/// Kind of payload being exchanged between peers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PeerPayloadKind {
    PersonalMesh,
    SharedFolder,
    DropBox,
    AskMyBot,
    PeerBackup,
    Calls,
    PeerCache,
}

impl PeerPayloadKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PersonalMesh => "personal-mesh",
            Self::SharedFolder => "shared-folder",
            Self::DropBox => "drop-box",
            Self::AskMyBot => "ask-my-bot",
            Self::PeerBackup => "peer-backup",
            Self::Calls => "calls",
            Self::PeerCache => "peer-cache",
        }
    }

    pub fn all() -> &'static [PeerPayloadKind] {
        &[
            Self::PersonalMesh,
            Self::SharedFolder,
            Self::DropBox,
            Self::AskMyBot,
            Self::PeerBackup,
            Self::Calls,
            Self::PeerCache,
        ]
    }
}

/// A payload envelope exchanged between two peers. The body is opaque to the
/// relay and only the recipient can decrypt it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerPayload {
    pub payload_id: Uuid,
    pub kind: PeerPayloadKind,
    pub from: Uuid,
    pub to: Uuid,
    /// Opaque, end-to-end encrypted body (base64 or raw bytes depending on
    /// transport). Never inspected by intermediate nodes.
    pub body: String,
    pub created_at: u64,
    /// Optional expiry (unix seconds). The relay drops the payload after this.
    #[serde(default)]
    pub expires_at: Option<u64>,
}

impl PeerPayload {
    pub fn new(
        kind: PeerPayloadKind,
        from: Uuid,
        to: Uuid,
        body: impl Into<String>,
        created_at: u64,
    ) -> Self {
        Self {
            payload_id: Uuid::new_v4(),
            kind,
            from,
            to,
            body: body.into(),
            created_at,
            expires_at: None,
        }
    }

    pub fn with_expiry(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at.map(|expiry| now >= expiry).unwrap_or(false)
    }
}

/// In-memory store for pending payloads. Nodes do not retain user content
/// beyond the configured TTL.
pub struct PeerPayloadStore {
    payloads: std::sync::RwLock<std::collections::HashMap<Uuid, PeerPayload>>,
    ttl_seconds: u64,
}

impl PeerPayloadStore {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            payloads: std::sync::RwLock::new(std::collections::HashMap::new()),
            ttl_seconds: ttl_seconds.max(1),
        }
    }

    pub fn ttl_seconds(&self) -> u64 {
        self.ttl_seconds
    }

    pub fn enqueue(&self, payload: PeerPayload) -> Result<PeerPayload, String> {
        let mut payloads = self
            .payloads
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        prune_payloads(&mut payloads, unix_now(), self.ttl_seconds);
        if payloads.len() >= 1024 {
            return Err("payload store is full".to_string());
        }
        payloads.insert(payload.payload_id, payload.clone());
        Ok(payload)
    }

    pub fn dequeue(&self, payload_id: Uuid) -> Option<PeerPayload> {
        let mut payloads = self
            .payloads
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        payloads.remove(&payload_id)
    }

    pub fn pending_for(&self, recipient: Uuid) -> Vec<PeerPayload> {
        let mut payloads = self
            .payloads
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        prune_payloads(&mut payloads, unix_now(), self.ttl_seconds);
        let mut list: Vec<PeerPayload> = payloads
            .values()
            .filter(|payload| payload.to == recipient)
            .cloned()
            .collect();
        list.sort_by_key(|payload| payload.created_at);
        list
    }

    pub fn len(&self) -> usize {
        self.payloads
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn prune_payloads(
    payloads: &mut std::collections::HashMap<Uuid, PeerPayload>,
    now: u64,
    ttl_seconds: u64,
) {
    payloads.retain(|_, payload| {
        if payload.is_expired(now) {
            return false;
        }
        now.saturating_sub(payload.created_at) <= ttl_seconds
    });
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

static GLOBAL_STORE: OnceLock<PeerPayloadStore> = OnceLock::new();

pub fn global_store() -> &'static PeerPayloadStore {
    GLOBAL_STORE.get_or_init(|| PeerPayloadStore::new(300))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_round_trip() {
        for kind in PeerPayloadKind::all() {
            let serialized = serde_json::to_string(kind).expect("serialize");
            let parsed: PeerPayloadKind = serde_json::from_str(&serialized).expect("parse");
            assert_eq!(parsed, *kind);
        }
    }

    #[test]
    fn payload_expiry() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let payload = PeerPayload::new(PeerPayloadKind::DropBox, from, to, "body", 100)
            .with_expiry(150);
        assert!(!payload.is_expired(149));
        assert!(payload.is_expired(150));
    }

    #[test]
    fn store_enqueue_and_pending() {
        let from = Uuid::new_v4();
        let to = Uuid::new_v4();
        let store = PeerPayloadStore::new(60);
        let payload = PeerPayload::new(PeerPayloadKind::AskMyBot, from, to, "hi", 1000);
        store.enqueue(payload.clone()).expect("enqueue");
        assert_eq!(store.len(), 1);
        let pending = store.pending_for(to);
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].payload_id, payload.payload_id);
    }
}
