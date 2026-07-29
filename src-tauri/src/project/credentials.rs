//! Credential hashing and verification.
//!
//! New secrets are stored as Argon2id PHC strings with a per-secret random salt.
//! Project files written before this module used a single unsalted-per-user
//! SHA-256 digest with one global salt; those digests are still *verifiable* so
//! existing projects keep working, but they are never produced again and every
//! successful verification reports that the record needs re-hashing.

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString};
use argon2::{Argon2, PasswordVerifier};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Marker prefix of a PHC-encoded Argon2 hash.
const PHC_PREFIX: &str = "$argon2";

/// Hash a password or PIN with Argon2id and a fresh random salt.
pub fn hash_secret(secret: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("password hashing failed: {e}"))
}

/// Legacy digest kept only so that projects created before Argon2 can still log in.
fn legacy_digest(secret: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}

/// Outcome of verifying a stored credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verification {
    /// Secret does not match the stored record.
    Rejected,
    /// Secret matches a current Argon2id record.
    Accepted,
    /// Secret matches a legacy SHA-256 record that must be upgraded.
    AcceptedNeedsRehash,
}

impl Verification {
    pub fn is_accepted(&self) -> bool {
        !matches!(self, Verification::Rejected)
    }
}

/// Verify `secret` against `stored`, accepting both Argon2id and legacy records.
///
/// The comparison is constant-time for the legacy path; Argon2 verification is
/// constant-time by construction.
pub fn verify_secret(secret: &str, stored: &str, legacy_salt: &str) -> Verification {
    if stored.starts_with(PHC_PREFIX) {
        let Ok(parsed) = PasswordHash::new(stored) else {
            return Verification::Rejected;
        };
        return match Argon2::default().verify_password(secret.as_bytes(), &parsed) {
            Ok(()) => Verification::Accepted,
            Err(_) => Verification::Rejected,
        };
    }

    let candidate = legacy_digest(secret, legacy_salt);
    if candidate.as_bytes().ct_eq(stored.as_bytes()).into() {
        Verification::AcceptedNeedsRehash
    } else {
        Verification::Rejected
    }
}

/// True when the stored record still uses the superseded SHA-256 format.
#[cfg(test)]
fn is_legacy(stored: &str) -> bool {
    !stored.starts_with(PHC_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argon2_roundtrip_accepts_only_the_right_secret() {
        let stored = hash_secret("correct horse battery").expect("hash");
        assert!(stored.starts_with(PHC_PREFIX));
        assert_eq!(
            verify_secret("correct horse battery", &stored, "unused"),
            Verification::Accepted
        );
        assert_eq!(
            verify_secret("wrong", &stored, "unused"),
            Verification::Rejected
        );
    }

    #[test]
    fn two_hashes_of_the_same_secret_use_different_salts() {
        let a = hash_secret("same-secret").expect("hash");
        let b = hash_secret("same-secret").expect("hash");
        assert_ne!(a, b, "per-secret random salt must be used");
    }

    #[test]
    fn legacy_sha256_records_verify_but_are_flagged_for_rehash() {
        let legacy = legacy_digest("admin123", "proscada_salt");
        assert!(is_legacy(&legacy));
        assert_eq!(
            verify_secret("admin123", &legacy, "proscada_salt"),
            Verification::AcceptedNeedsRehash
        );
        assert_eq!(
            verify_secret("admin124", &legacy, "proscada_salt"),
            Verification::Rejected
        );
    }

    #[test]
    fn malformed_phc_record_is_rejected_instead_of_panicking() {
        assert_eq!(
            verify_secret("anything", "$argon2id$not-a-real-hash", "salt"),
            Verification::Rejected
        );
    }
}
