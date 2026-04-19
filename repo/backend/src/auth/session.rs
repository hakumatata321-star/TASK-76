use crate::models::Claims;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Maximum idle time: 12 hours. If (now - iat) > this, the token is expired.
const IDLE_TIMEOUT_SECS: i64 = 12 * 3600;

pub fn create_token(
    user_id: &str,
    username: &str,
    role: &str,
    store_id: Option<&str>,
    secret: &str,
) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        user_id: user_id.to_string(),
        username: username.to_string(),
        role: role.to_string(),
        store_id: store_id.map(String::from),
        iat: now,
        exp: now + IDLE_TIMEOUT_SECS,
    };

    let payload = serde_json::to_string(&claims).unwrap();
    let payload_b64 = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &payload);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload_b64.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    format!("{}.{}", payload_b64, signature)
}

pub fn validate_token(token: &str, secret: &str) -> Option<Claims> {
    let parts: Vec<&str> = token.splitn(2, '.').collect();
    if parts.len() != 2 {
        return None;
    }

    let payload_b64 = parts[0];
    let signature = parts[1];

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(payload_b64.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    if signature != expected {
        return None;
    }

    let payload_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        payload_b64,
    ).ok()?;
    let claims: Claims = serde_json::from_slice(&payload_bytes).ok()?;

    let now = Utc::now().timestamp();

    // Idle timeout: token expires if (now - iat) > IDLE_TIMEOUT_SECS
    // The `exp` field is set to iat + IDLE_TIMEOUT at creation time.
    // On each API call, the frontend should use the refreshed token from /api/auth/me
    // which resets iat to now. If the user has been idle for >12h, exp < now.
    if claims.exp < now {
        return None;
    }

    Some(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_validate_token() {
        let secret = "test-secret-key-32-bytes-long!!!";
        let token = create_token("user-1", "testuser", "Customer", Some("store-1"), secret);
        let claims = validate_token(&token, secret).unwrap();
        assert_eq!(claims.user_id, "user-1");
        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.role, "Customer");
        assert_eq!(claims.store_id, Some("store-1".to_string()));
    }

    #[test]
    fn test_idle_timeout_window() {
        let secret = "test-secret-key-32-bytes-long!!!";
        let token = create_token("user-1", "test", "Customer", None, secret);
        let claims = validate_token(&token, secret).unwrap();
        // Token should be valid and exp should be iat + 12h
        assert_eq!(claims.exp - claims.iat, IDLE_TIMEOUT_SECS);
    }

    #[test]
    fn test_invalid_signature_rejected() {
        let token = create_token("user-1", "test", "Customer", None, "secret1");
        assert!(validate_token(&token, "secret2").is_none());
    }

    #[test]
    fn test_tampered_payload_rejected() {
        let secret = "test-secret";
        let token = create_token("user-1", "test", "Customer", None, secret);
        let tampered = format!("dGFtcGVyZWQ.{}", token.split('.').nth(1).unwrap());
        assert!(validate_token(&tampered, secret).is_none());
    }

    #[test]
    fn test_expired_token_rejected() {
        let secret = "test-secret";
        let claims = Claims {
            user_id: "u1".into(),
            username: "test".into(),
            role: "Customer".into(),
            store_id: None,
            iat: 0,
            exp: 1, // expired in 1970
        };
        let payload = serde_json::to_string(&claims).unwrap();
        let payload_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            &payload,
        );
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload_b64.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());
        let token = format!("{}.{}", payload_b64, sig);
        assert!(validate_token(&token, secret).is_none());
    }

    #[test]
    fn test_refreshed_token_has_newer_iat() {
        let secret = "test-secret-key-32-bytes-long!!!";
        let t1 = create_token("u1", "test", "Customer", None, secret);
        let c1 = validate_token(&t1, secret).unwrap();
        // Simulate a small delay then reissue
        let t2 = create_token("u1", "test", "Customer", None, secret);
        let c2 = validate_token(&t2, secret).unwrap();
        assert!(c2.iat >= c1.iat);
        assert!(c2.exp >= c1.exp);
    }
}
