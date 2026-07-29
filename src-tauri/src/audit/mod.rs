//! Hash-chained append-only audit trail with durable JSONL persistence.
//!
//! Two invariants matter here:
//!
//! 1. The in-memory window is bounded, but trimming it must never make
//!    [`AuditLog::verify_chain`] report a forged chain. Trimming therefore moves
//!    the verification anchor forward instead of resetting it to `GENESIS`.
//! 2. A failure to persist is surfaced through [`AuditLog::status`] rather than
//!    being swallowed, so the UI can show that the trail is degraded.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Anchor value of an audit chain that has never been trimmed.
pub const GENESIS: &str = "GENESIS";

/// Entries kept in memory before the oldest ones are trimmed.
const MEMORY_LIMIT: usize = 5000;
/// Entries retained after a trim.
const MEMORY_RETAIN: usize = 4000;

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

impl AuditEntry {
    fn digest(&self) -> String {
        entry_digest(
            &self.id,
            self.ts,
            &self.actor,
            &self.role,
            &self.action,
            &self.detail,
            &self.prev_hash,
        )
    }
}

fn entry_digest(
    id: &str,
    ts: DateTime<Utc>,
    actor: &str,
    role: &str,
    action: &str,
    detail: &str,
    prev_hash: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    hasher.update(ts.to_rfc3339().as_bytes());
    hasher.update(actor.as_bytes());
    hasher.update(role.as_bytes());
    hasher.update(action.as_bytes());
    hasher.update(detail.as_bytes());
    hasher.update(prev_hash.as_bytes());
    hex::encode(hasher.finalize())
}

/// Reported state of the audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatus {
    /// Chain integrity of the entries currently held in memory.
    pub chain_ok: bool,
    /// Absolute path of the durable log, when persistence is active.
    pub sink_path: Option<String>,
    /// True when every entry appended so far reached the durable log.
    pub persisted: bool,
    /// Last persistence error, if any.
    pub last_error: Option<String>,
    /// Number of entries currently retained in memory.
    pub in_memory: usize,
    /// Total number of entries appended since process start.
    pub appended: u64,
    /// Number of append operations that could not be durably synchronized.
    pub failed_appends: u64,
}

struct Inner {
    entries: Vec<AuditEntry>,
    /// Hash of the newest entry that has been trimmed from `entries`.
    anchor: String,
    sink: Option<PathBuf>,
    last_error: Option<String>,
    appended: u64,
    failed_appends: u64,
}

pub struct AuditLog {
    inner: Mutex<Inner>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                entries: Vec::new(),
                anchor: GENESIS.to_string(),
                sink: None,
                last_error: None,
                appended: 0,
                failed_appends: 0,
            }),
        }
    }

    /// Attach a durable JSONL sink, replaying an existing file so the hash chain
    /// continues across restarts. Returns the number of entries recovered.
    pub fn attach_sink(&self, path: &Path) -> Result<usize, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("audit directory {}: {e}", parent.display()))?;
        }
        let recovered = if path.exists() {
            let file =
                File::open(path).map_err(|e| format!("audit log {}: {e}", path.display()))?;
            let mut restored = Vec::new();
            for line in BufReader::new(file).lines() {
                let line = line.map_err(|e| format!("audit log read: {e}"))?;
                if line.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<AuditEntry>(&line) {
                    Ok(entry) => restored.push(entry),
                    Err(e) => return Err(format!("audit log is corrupt: {e}")),
                }
            }
            restored
        } else {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .map_err(|e| format!("audit log {}: {e}", path.display()))?;
            Vec::new()
        };

        if !verify_entries(&recovered, GENESIS) {
            return Err("audit log hash chain verification failed".into());
        }

        let mut guard = self.inner.lock();
        let pending = std::mem::take(&mut guard.entries);
        let count = recovered.len();
        if recovered.len() > MEMORY_RETAIN {
            let drain = recovered.len() - MEMORY_RETAIN;
            guard.anchor = recovered[drain - 1].hash.clone();
            guard.entries = recovered[drain..].to_vec();
        } else {
            guard.anchor = GENESIS.to_string();
            guard.entries = recovered;
        }
        guard.sink = Some(path.to_path_buf());
        guard.last_error = None;
        guard.failed_appends = 0;
        drop(guard);

        // Re-link entries appended before the sink existed so the chain stays valid.
        for entry in pending {
            self.append_required(&entry.actor, &entry.role, &entry.action, &entry.detail)?;
        }
        Ok(count)
    }

    pub fn append(&self, actor: &str, role: &str, action: &str, detail: &str) -> AuditEntry {
        let mut guard = self.inner.lock();
        let prev_hash = guard
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| guard.anchor.clone());
        let id = Uuid::new_v4().to_string();
        let ts = Utc::now();
        let hash = entry_digest(&id, ts, actor, role, action, detail, &prev_hash);
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

        if guard.failed_appends == 0 {
            if let Some(path) = guard.sink.clone() {
                match persist(&path, &entry) {
                    Ok(()) => {}
                    Err(e) => {
                        guard.last_error = Some(e);
                        guard.failed_appends = guard.failed_appends.saturating_add(1);
                    }
                }
            }
        }

        guard.entries.push(entry.clone());
        guard.appended = guard.appended.saturating_add(1);
        if guard.entries.len() > MEMORY_LIMIT {
            let drain = guard.entries.len() - MEMORY_RETAIN;
            // The newest trimmed entry becomes the anchor so `verify_chain`
            // keeps validating the retained window instead of failing forever.
            guard.anchor = guard.entries[drain - 1].hash.clone();
            guard.entries.drain(0..drain);
        }
        entry
    }

    /// Append an event that must be durable before a protected mutation may
    /// proceed. Once durability or chain integrity degrades, required appends
    /// fail closed until the process is restarted and the sink is recovered.
    pub fn append_required(
        &self,
        actor: &str,
        role: &str,
        action: &str,
        detail: &str,
    ) -> Result<AuditEntry, String> {
        let mut guard = self.inner.lock();
        let path = guard
            .sink
            .clone()
            .ok_or("Durable audit sink is not available")?;
        if guard.failed_appends > 0 {
            return Err(guard
                .last_error
                .clone()
                .unwrap_or_else(|| "Durable audit trail is degraded".into()));
        }
        if !verify_entries(&guard.entries, &guard.anchor) {
            let error = "Audit chain integrity check failed".to_string();
            guard.last_error = Some(error.clone());
            guard.failed_appends = guard.failed_appends.saturating_add(1);
            return Err(error);
        }

        let prev_hash = guard
            .entries
            .last()
            .map(|entry| entry.hash.clone())
            .unwrap_or_else(|| guard.anchor.clone());
        let id = Uuid::new_v4().to_string();
        let ts = Utc::now();
        let hash = entry_digest(&id, ts, actor, role, action, detail, &prev_hash);
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
        if let Err(error) = persist(&path, &entry) {
            guard.last_error = Some(error.clone());
            guard.failed_appends = guard.failed_appends.saturating_add(1);
            guard.entries.push(entry);
            guard.appended = guard.appended.saturating_add(1);
            return Err(error);
        }
        guard.entries.push(entry.clone());
        guard.appended = guard.appended.saturating_add(1);
        if guard.entries.len() > MEMORY_LIMIT {
            let drain = guard.entries.len() - MEMORY_RETAIN;
            guard.anchor = guard.entries[drain - 1].hash.clone();
            guard.entries.drain(0..drain);
        }
        Ok(entry)
    }

    pub fn list(&self, limit: usize) -> Vec<AuditEntry> {
        let guard = self.inner.lock();
        let n = guard.entries.len().min(limit);
        guard.entries[guard.entries.len().saturating_sub(n)..].to_vec()
    }

    pub fn verify_chain(&self) -> bool {
        let guard = self.inner.lock();
        verify_entries(&guard.entries, &guard.anchor)
    }

    pub fn status(&self) -> AuditStatus {
        let chain_ok = self.verify_chain();
        let guard = self.inner.lock();
        AuditStatus {
            chain_ok,
            sink_path: guard.sink.as_ref().map(|p| p.display().to_string()),
            persisted: guard.sink.is_some() && guard.last_error.is_none(),
            last_error: guard.last_error.clone(),
            in_memory: guard.entries.len(),
            appended: guard.appended,
            failed_appends: guard.failed_appends,
        }
    }

    /// Operator-visible health omits the local filesystem path.
    pub fn status_redacted(&self) -> AuditStatus {
        let mut status = self.status();
        status.sink_path = None;
        status
    }
}

fn verify_entries(entries: &[AuditEntry], anchor: &str) -> bool {
    let mut prev = anchor.to_string();
    for entry in entries {
        if entry.prev_hash != prev || entry.digest() != entry.hash {
            return false;
        }
        prev = entry.hash.clone();
    }
    true
}

fn persist(path: &Path, entry: &AuditEntry) -> Result<(), String> {
    let line = serde_json::to_string(entry).map_err(|e| format!("audit encode: {e}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("audit open {}: {e}", path.display()))?;
    file.write_all(line.as_bytes())
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.flush())
        .and_then(|()| file.sync_data())
        .map_err(|e| format!("audit write {}: {e}", path.display()))
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_verifies_for_a_fresh_log() {
        let log = AuditLog::new();
        for i in 0..10 {
            log.append("tester", "engineer", "tag.write", &format!("t{i}=1"));
        }
        assert!(log.verify_chain());
        assert_eq!(log.list(5).len(), 5);
    }

    #[test]
    fn chain_still_verifies_after_the_memory_window_is_trimmed() {
        let log = AuditLog::new();
        for i in 0..(MEMORY_LIMIT + 250) {
            log.append("tester", "engineer", "tag.write", &format!("t{i}=1"));
        }
        let status = log.status();
        assert!(status.in_memory <= MEMORY_LIMIT);
        assert!(
            log.verify_chain(),
            "trimming the in-memory window must not report a forged chain"
        );
        assert_ne!(status.appended, 0);
    }

    #[test]
    fn tampering_with_a_retained_entry_is_detected() {
        let log = AuditLog::new();
        log.append("a", "viewer", "app.start", "one");
        log.append("b", "viewer", "app.start", "two");
        {
            let mut guard = log.inner.lock();
            guard.entries[1].detail = "forged".into();
        }
        assert!(!log.verify_chain());
    }

    #[test]
    fn entries_survive_a_restart_through_the_jsonl_sink() {
        let dir = std::env::temp_dir().join(format!("proscada-audit-{}", Uuid::new_v4()));
        let path = dir.join("audit.jsonl");

        let first = AuditLog::new();
        first.attach_sink(&path).expect("attach");
        first.append("op", "operator", "tag.write", "wt.sp=10");
        first.append("op", "operator", "tag.write", "wt.sp=11");

        let second = AuditLog::new();
        let recovered = second.attach_sink(&path).expect("reattach");
        assert_eq!(recovered, 2);
        assert!(second.verify_chain());
        second.append("op", "operator", "tag.write", "wt.sp=12");
        assert!(second.verify_chain());
        assert_eq!(second.list(10).len(), 3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn entries_appended_before_the_sink_are_relinked_not_lost() {
        let dir = std::env::temp_dir().join(format!("proscada-audit-{}", Uuid::new_v4()));
        let path = dir.join("audit.jsonl");

        let log = AuditLog::new();
        log.append("system", "system", "app.start", "core online");
        log.attach_sink(&path).expect("attach");

        assert!(log.verify_chain());
        assert_eq!(log.list(10).len(), 1);
        assert!(log.status().persisted);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_corrupt_recovered_chain_is_rejected_before_it_becomes_authoritative() {
        let dir = std::env::temp_dir().join(format!("proscada-audit-{}", Uuid::new_v4()));
        let path = dir.join("audit.jsonl");
        let first = AuditLog::new();
        first.attach_sink(&path).expect("attach");
        first.append("op", "operator", "tag.write", "one");
        first.append("op", "operator", "tag.write", "two");

        let body = std::fs::read_to_string(&path).expect("read audit");
        std::fs::write(&path, body.replace("\"two\"", "\"forged\"")).expect("forge test log");

        let recovered = AuditLog::new();
        let error = recovered
            .attach_sink(&path)
            .expect_err("forged chain must be rejected");
        assert!(error.contains("verification failed"), "unexpected: {error}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn required_append_fails_closed_without_a_sink_or_after_tampering() {
        let log = AuditLog::new();
        assert!(log
            .append_required("op", "operator", "tag.write_requested", "x=1")
            .is_err());

        let dir = std::env::temp_dir().join(format!("proscada-audit-{}", Uuid::new_v4()));
        let path = dir.join("audit.jsonl");
        log.attach_sink(&path).expect("attach");
        log.append("a", "viewer", "app.start", "one");
        log.inner.lock().entries[0].detail = "forged".into();
        assert!(log
            .append_required("op", "operator", "tag.write_requested", "x=1")
            .is_err());
        assert!(!log.status().persisted);
        assert!(log.status().failed_appends > 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn ordinary_appends_become_memory_only_after_first_persistence_failure() {
        let dir = std::env::temp_dir().join(format!("proscada-audit-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("dir");
        let log = AuditLog::new();
        {
            let mut guard = log.inner.lock();
            // Opening a directory as an append-only file deterministically fails.
            guard.sink = Some(dir.clone());
        }
        log.append("a", "operator", "one", "will fail");
        assert_eq!(log.status().failed_appends, 1);

        let would_be_recovery = dir.join("must-remain-empty.jsonl");
        log.inner.lock().sink = Some(would_be_recovery.clone());
        log.append("a", "operator", "two", "memory only");
        assert!(
            !would_be_recovery.exists(),
            "a degraded logger must not create a durable chain with a missing predecessor"
        );
        assert_eq!(log.list(10).len(), 2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
