use fleetreserve_backend::auth::{csrf, password, session};

#[test]
fn unit_auth_hash_and_verify() {
    let hash = password::hash_password("pass123").unwrap();
    assert!(hash.starts_with("$argon2id$"));
    assert!(password::verify_password("pass123", &hash));
    assert!(!password::verify_password("bad", &hash));
}

#[test]
fn unit_auth_session_and_csrf() {
    let token = session::create_token(
        "u1",
        "alice",
        "Customer",
        None,
        "test-hmac-secret-32-bytes-minimum!!",
    );
    let claims = session::validate_token(&token, "test-hmac-secret-32-bytes-minimum!!").unwrap();
    assert_eq!(claims.user_id, "u1");

    let c = csrf::generate_csrf_token();
    assert!(csrf::validate_csrf_token(&c, &c));
    assert!(!csrf::validate_csrf_token(&c, "wrong"));
}
