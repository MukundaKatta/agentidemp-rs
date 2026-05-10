# agentidemp

[![crates.io](https://img.shields.io/crates/v/agentidemp.svg)](https://crates.io/crates/agentidemp)
[![docs.rs](https://docs.rs/agentidemp/badge.svg)](https://docs.rs/agentidemp)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Idempotency keys for LLM agent retries. Deterministic content-derived keys (UUIDv5 or sha256-hex) so retries dedupe at the provider or your dedup layer.

```toml
[dependencies]
agentidemp = "0.1"
```

## Why

When an agent retries — on a 429, a timeout, or a cachebench miss-aware policy — the retry needs the **same** idempotency key as the first attempt. Otherwise your dedup layer treats it as a new request and you double-bill or double-dispatch. `agentidemp` derives the key from the content, so retries get the same key automatically.

## Quick start

```rust
use agentidemp::{sha256_hex, uuid_v5, random, NAMESPACE_ANTHROPIC};

let body = serde_json::json!({
    "model": "claude-sonnet-4-20250514",
    "messages": [{"role": "user", "content": "hi"}]
});
let bytes = serde_json::to_vec(&body).unwrap();

// Short hex form (35 chars total, "ik_" prefix + 32 hex):
let key1 = sha256_hex(&bytes);
// → "ik_a3f9c1d8b2e7f046189f43a2b8e7c106"

// UUID v5 form (for `Idempotency-Key: <uuid>` headers):
let key2 = uuid_v5(&NAMESPACE_ANTHROPIC, &bytes);

// Random fresh key when you don't have content to derive from:
let key3 = random();
```

## Composing with `cachebench` retries

`cachebench`'s miss-aware policy retries on a silent cache miss. Pair with `agentidemp` so the retry is recognized:

```rust,ignore
use cachebench::{CacheTracker, CachePolicy, Provider};
use agentidemp::sha256_hex;

let body = serde_json::to_vec(&payload).unwrap();
let key = sha256_hex(&body);
// add `Idempotency-Key: ${key}` to your request headers
// retries built by CachePolicy::miss_aware reuse the same body → same key
```

## Scoped keys

If the same prompt body might recur across tenants and you want per-tenant dedup, prefix with a scope string:

```rust
use agentidemp::scoped_sha256_hex;

let key = scoped_sha256_hex("user-42", &body_bytes);
```

The scope and content are separated by a null byte so `"ab" + "c"` doesn't collide with `"a" + "bc"`.

## What it doesn't do

- Doesn't make HTTP calls. Returns a string; you set the header.
- Doesn't cache responses. That's a different layer.
- Doesn't track which keys you've used. Stateless by design.

## License

MIT
