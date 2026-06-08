//! Idempotency keys for LLM agent retries.
//!
//! When an agent retries a request — whether due to a 429, a timeout, or a
//! cache miss-aware policy ([cachebench](https://crates.io/crates/cachebench))
//! — the retry should carry the **same** idempotency key as the first
//! attempt so the server (or your dedup layer) recognizes it as a retry,
//! not a new request.
//!
//! `agentidemp` derives stable keys from the request content. Two flavors:
//!
//! - [`sha256_hex`] — short hex digest of the content, prefixed with `ik_`.
//! - [`uuid_v5`] — namespaced deterministic UUID, suitable for headers
//!   that expect UUID format (e.g. `Idempotency-Key: <uuid>`).
//!
//! Plus [`random`] for the "I want a fresh key" case.
//!
//! # Quick start
//!
//! ```
//! use agentidemp::{sha256_hex, uuid_v5, NAMESPACE_ANTHROPIC};
//!
//! let body = serde_json::json!({"model": "claude", "messages": [{"role": "user", "content": "hi"}]});
//! let bytes = serde_json::to_vec(&body).unwrap();
//!
//! let k = sha256_hex(&bytes);
//! assert!(k.starts_with("ik_"));
//! assert_eq!(k.len(), 3 + 32); // ik_ + 32 hex chars
//!
//! let u = uuid_v5(&NAMESPACE_ANTHROPIC, &bytes);
//! assert_eq!(u.to_string().len(), 36);
//! ```
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use std::fmt::Write as _;

use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Format the first 16 bytes of `digest` as the `ik_`-prefixed hex string
/// returned by [`sha256_hex`] and [`scoped_sha256_hex`].
fn ik_hex(digest: &[u8]) -> String {
    // "ik_" (3) + 16 bytes * 2 hex chars (32) = 35 chars.
    let mut s = String::with_capacity(35);
    s.push_str("ik_");
    for b in &digest[..16] {
        // Writing into the pre-sized buffer avoids a per-byte allocation.
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// A stable namespace UUID for Anthropic-bound requests. Use as the first
/// arg to [`uuid_v5`] to scope your keys.
pub const NAMESPACE_ANTHROPIC: Uuid = Uuid::from_bytes([
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
]);
// (NB: This is the standard DNS namespace UUID; we re-export it as a
// convenient anchor for users who don't want to think about namespaces.)

/// Compute a 16-byte sha256 prefix of `content`, hex-encoded, with the
/// `ik_` marker prefix. Useful for `Idempotency-Key` HTTP headers that
/// don't require a UUID format.
///
/// Returns a 35-character string (3 prefix + 32 hex).
///
/// ```
/// # use agentidemp::sha256_hex;
/// // sha256("abc") = ba7816bf8f01cfea414140de5dae2223...; first 16 bytes are hex-encoded.
/// assert_eq!(sha256_hex(b"abc"), "ik_ba7816bf8f01cfea414140de5dae2223");
/// ```
#[must_use]
pub fn sha256_hex(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let digest = hasher.finalize();
    // First 16 bytes of the digest = 32 hex chars
    ik_hex(&digest)
}

/// Compute a deterministic [UUID v5][rfc] for `content` under `namespace`.
///
/// Same content + same namespace ⇒ same UUID, every time. Use this when
/// the destination expects a UUID-shaped idempotency key.
///
/// [rfc]: https://datatracker.ietf.org/doc/html/rfc4122#section-4.3
#[must_use]
pub fn uuid_v5(namespace: &Uuid, content: &[u8]) -> Uuid {
    Uuid::new_v5(namespace, content)
}

/// Generate a random UUID v4. For when you don't want determinism — e.g.
/// the very first attempt where you have no prior key to reuse.
#[must_use]
pub fn random() -> Uuid {
    Uuid::new_v4()
}

/// Combine `scope` and `content` into a single hashed key. Useful when
/// the same content might recur across users/tenants and you want
/// per-scope deduplication.
///
/// Equivalent to `sha256_hex` over `scope || 0x00 || content`.
#[must_use]
pub fn scoped_sha256_hex(scope: &str, content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(scope.as_bytes());
    hasher.update([0u8]); // separator so "ab"+"c" != "a"+"bc"
    hasher.update(content);
    let digest = hasher.finalize();
    ik_hex(&digest)
}
