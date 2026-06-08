use agentidemp::{random, scoped_sha256_hex, sha256_hex, uuid_v5, NAMESPACE_ANTHROPIC};

#[test]
fn sha256_hex_format() {
    let k = sha256_hex(b"hello");
    assert!(k.starts_with("ik_"));
    assert_eq!(k.len(), 35);
    // hex characters only after the prefix
    assert!(k[3..].chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn sha256_hex_is_deterministic() {
    assert_eq!(sha256_hex(b"x"), sha256_hex(b"x"));
}

#[test]
fn sha256_hex_different_for_different_content() {
    assert_ne!(sha256_hex(b"a"), sha256_hex(b"b"));
}

#[test]
fn sha256_hex_known_vector() {
    // sha256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
    // First 16 bytes (32 hex chars) = ba7816bf8f01cfea414140de5dae2223
    assert_eq!(sha256_hex(b"abc"), "ik_ba7816bf8f01cfea414140de5dae2223");
}

#[test]
fn readme_quick_start_vector() {
    // Pins the exact `sha256_hex` output shown in the README quick start so the
    // documented value can't silently drift away from the implementation.
    let body = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "messages": [{"role": "user", "content": "hi"}]
    });
    let bytes = serde_json::to_vec(&body).unwrap();
    assert_eq!(sha256_hex(&bytes), "ik_5c181ca05655562a20cd9639a1bea983");
}

#[test]
fn uuid_v5_is_deterministic() {
    let a = uuid_v5(&NAMESPACE_ANTHROPIC, b"some-payload");
    let b = uuid_v5(&NAMESPACE_ANTHROPIC, b"some-payload");
    assert_eq!(a, b);
}

#[test]
fn uuid_v5_different_for_different_content() {
    let a = uuid_v5(&NAMESPACE_ANTHROPIC, b"x");
    let b = uuid_v5(&NAMESPACE_ANTHROPIC, b"y");
    assert_ne!(a, b);
}

#[test]
fn uuid_v5_marks_v5_bits() {
    let u = uuid_v5(&NAMESPACE_ANTHROPIC, b"x");
    assert_eq!(u.get_version_num(), 5);
}

#[test]
fn random_is_v4_and_unique() {
    let a = random();
    let b = random();
    assert_eq!(a.get_version_num(), 4);
    assert_ne!(a, b);
}

#[test]
fn scoped_changes_with_scope() {
    let a = scoped_sha256_hex("user-1", b"payload");
    let b = scoped_sha256_hex("user-2", b"payload");
    let same = scoped_sha256_hex("user-1", b"payload");
    assert_ne!(a, b);
    assert_eq!(a, same);
}

#[test]
fn scoped_separator_prevents_collisions() {
    // "ab" + "c" must not collide with "a" + "bc"
    let a = scoped_sha256_hex("ab", b"c");
    let b = scoped_sha256_hex("a", b"bc");
    assert_ne!(a, b);
}

#[test]
fn scoped_sha256_hex_format() {
    let k = scoped_sha256_hex("user-1", b"payload");
    assert!(k.starts_with("ik_"));
    assert_eq!(k.len(), 35);
    assert!(k[3..].chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn scoped_known_vector() {
    // Equivalent to sha256("user-1" || 0x00 || "payload"), first 16 bytes.
    let k = scoped_sha256_hex("user-1", b"payload");
    let manual = sha256_hex(b"user-1\0payload");
    assert_eq!(k, manual);
}

#[test]
fn uuid_v5_depends_on_namespace() {
    // A different namespace must yield a different UUID for the same content.
    let other_ns = uuid::Uuid::from_bytes([1u8; 16]);
    let other = uuid_v5(&other_ns, b"x");
    let anthropic = uuid_v5(&NAMESPACE_ANTHROPIC, b"x");
    assert_ne!(other, anthropic);
}
