//! Hash-chained append-only audit trail.

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub ts: DateTime<Utc>,
    pub actor: String,
    pub role: String,
    pub action: String,
    pub detail: String,
    pub prev_hash: String,
    pub hash: String,
}

pub struct AuditLog {
    entries: Mutex<Vec<AuditEntry>>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
        }
    }

    pub fn append(&self, actor: &str, role: &str, action: &str, detail: &str) -> AuditEntry {
        let mut guard = self.entries.lock();
        let prev_hash = guard
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| "GENESIS".into());
        let id = Uuid::new_v4().to_string();
        let ts = Utc::now();
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(ts.to_rfc3339().as_bytes());
        hasher.update(actor.as_bytes());
        hasher.update(role.as_bytes());
        hasher.update(action.as_bytes());
        hasher.update(detail.as_bytes());
        hasher.update(prev_hash.as_bytes());
        let hash = hex::encode(hasher.finalize());
        let entry = AuditEntry {
            id,
            ts,
            actor: actor.into(),
            role: role.into(),
            action: action.into(),
            detail: detail.into(),
            prev_hash,
            hash,
        };
        guard.push(entry.clone());
        // Cap memory
        if guard.len() > 5000 {
            let drain = guard.len() - 4000;
            guard.drain(0..drain);
        }
        entry
    }

    pub fn list(&self, limit: usize) -> Vec<AuditEntry> {
        let guard = self.entries.lock();
        let n = guard.len().min(limit);
        guard[guard.len().saturating_sub(n)..].to_vec()
    }

    pub fn verify_chain(&self) -> bool {
        let guard = self.entries.lock();
        let mut prev = "GENESIS".to_string();
        for e in guard.iter() {
            if e.prev_hash != prev {
                return false;
            }
            let mut hasher = Sha256::new();
            hasher.update(e.id.as_bytes());
            hasher.update(e.ts.to_rfc3339().as_bytes());
            hasher.update(e.actor.as_bytes());
            hasher.update(e.role.as_bytes());
            hasher.update(e.action.as_bytes());
            hasher.update(e.detail.as_bytes());
            hasher.update(e.prev_hash.as_bytes());
            let h = hex::encode(hasher.finalize());
            if h != e.hash {
                return false;
            }
            prev = e.hash.clone();
        }
        true
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}
