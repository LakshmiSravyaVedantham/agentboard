use agentboard::auth::{generate_pairing_code, create_token, verify_token};

#[test]
fn test_pairing_code_is_6_digits() {
    let code = generate_pairing_code();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_token_roundtrip() {
    let secret = "test-secret-key-for-jwt";
    let token = create_token(secret).unwrap();
    assert!(verify_token(&token, secret).is_ok());
}

#[test]
fn test_invalid_token_rejected() {
    let secret = "test-secret";
    assert!(verify_token("garbage.token.here", secret).is_err());
}
