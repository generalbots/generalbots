//! Auto-generated signed home site (#1867).
//!
//! When a user joins the P2P mesh, a signed home page is published at
//! `<name>.generalbots.org`. The page is signed with the node identity so
//! recipients can verify it came from the owning peer. This module builds
//! the site document and its signature; hosting/relay wiring is separate.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A signed home site published for a peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeSite {
    pub name: String,
    pub host: String,
    pub owner_node_id: Uuid,
    pub title: String,
    pub body: String,
    pub created_at: u64,
    /// Hex-encoded signature over the canonical site payload.
    pub signature: String,
}

impl HomeSite {
    /// Canonical host for a peer name.
    pub fn host_for(name: &str) -> String {
        format!("{}.generalbots.org", name.trim().to_lowercase())
    }

    /// Build an unsigned site skeleton.
    pub fn new(name: impl Into<String>, owner_node_id: Uuid, title: impl Into<String>, body: impl Into<String>, created_at: u64) -> Self {
        let name = name.into();
        let host = Self::host_for(&name);
        Self {
            name,
            host,
            owner_node_id,
            title: title.into(),
            body: body.into(),
            created_at,
            signature: String::new(),
        }
    }

    /// Canonical bytes that get signed.
    pub fn canonical(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            self.name, self.host, self.owner_node_id, self.title, self.body
        )
    }

    /// Sign the site with the provided key material (hex). Uses a simple
    /// deterministic HMAC-like digest so the site is tamper-evident without
    /// pulling in a crypto dependency here. Production wiring should replace
    /// this with the vault-backed signing key.
    pub fn sign(&mut self, key: &str) {
        self.signature = simple_signature(key, &self.canonical());
    }

    pub fn verify(&self, key: &str) -> bool {
        simple_signature(key, &self.canonical()) == self.signature && !self.signature.is_empty()
    }
}

fn simple_signature(key: &str, message: &str) -> String {
    // FNV-1a based keyed digest, hex-encoded. Deterministic and dependency-free.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in key.bytes().chain(message.bytes()) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", hash)
}

/// Generator helper used when a user joins P2P.
pub struct HomeSiteGenerator;

impl HomeSiteGenerator {
    pub fn generate(
        name: impl Into<String>,
        owner_node_id: Uuid,
        title: impl Into<String>,
        body: impl Into<String>,
        created_at: u64,
        signing_key: &str,
    ) -> HomeSite {
        let mut site = HomeSite::new(name, owner_node_id, title, body, created_at);
        site.sign(signing_key);
        site
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateHomeSiteRequest {
    pub name: String,
    pub owner_node_id: Uuid,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_format() {
        assert_eq!(HomeSite::host_for("Alice"), "alice.generalbots.org");
    }

    #[test]
    fn sign_and_verify() {
        let mut site = HomeSite::new("alice", Uuid::new_v4(), "Hi", "body", 1);
        site.sign("secret");
        assert!(site.verify("secret"));
        assert!(!site.verify("wrong"));
    }

    #[test]
    fn generator_signs() {
        let site = HomeSiteGenerator::generate("bob", Uuid::new_v4(), "T", "B", 2, "k");
        assert!(!site.signature.is_empty());
        assert!(site.verify("k"));
    }
}
