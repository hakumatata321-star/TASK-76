pub fn mask_vin(vin: &str) -> String {
    if vin.len() <= 4 {
        return "*".repeat(vin.len());
    }
    let visible = &vin[vin.len() - 4..];
    format!("{}{}", "*".repeat(vin.len() - 4), visible)
}

pub fn mask_license_plate(plate: &str) -> String {
    if plate.len() <= 2 {
        return "*".repeat(plate.len());
    }
    let visible = &plate[plate.len() - 2..];
    format!("{}{}", "*".repeat(plate.len() - 2), visible)
}

pub fn mask_username(username: &str) -> String {
    if username.is_empty() {
        return String::new();
    }
    let first = &username[..1];
    format!("{}***", first)
}

pub fn mask_email(email: &str) -> String {
    if email.is_empty() {
        return String::new();
    }
    "****@****".to_string()
}

/// Redact a raw user UUID so it cannot be correlated across export payloads
/// while remaining distinguishable (keeps last 4 hex chars).
pub fn mask_user_id(id: &str) -> String {
    if id.len() <= 4 {
        return format!("usr-{}", "*".repeat(id.len()));
    }
    format!("usr-***-{}", &id[id.len() - 4..])
}

/// Produce a deterministic HMAC-SHA256 pseudonym for a user ID.
/// Stored in audit_log and non-FK ticket fields so the raw UUID never appears
/// at rest in those tables. Using the server HMAC key means the pseudonym is
/// consistent within one deployment but opaque to external parties.
pub fn pseudonymize_user_id(id: &str, hmac_key: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let key = if hmac_key.is_empty() { "default-pseudonym-key" } else { hmac_key };
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(id.as_bytes());
    let result = mac.finalize().into_bytes();
    format!("ph-{}", hex::encode(&result[..8]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_vin() {
        assert_eq!(mask_vin("1HGCM82633A123456"), "*************3456");
        // Length <= 4: mask entire value
        assert_eq!(mask_vin("ABCD"), "****");
        assert_eq!(mask_vin("AB"), "**");
    }

    #[test]
    fn test_mask_license_plate() {
        assert_eq!(mask_license_plate("ABC1234"), "*****34");
        // Length <= 2: mask entire value
        assert_eq!(mask_license_plate("AB"), "**");
    }

    #[test]
    fn test_mask_username() {
        assert_eq!(mask_username("johndoe"), "j***");
        assert_eq!(mask_username("a"), "a***");
        assert_eq!(mask_username(""), "");
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("john@example.com"), "****@****");
        assert_eq!(mask_email(""), "");
    }

    #[test]
    fn test_mask_user_id() {
        assert_eq!(mask_user_id("user-admin-001"), "usr-***--001");
        // len == 4: falls into the <=4 branch, fully masked
        assert_eq!(mask_user_id("abcd"), "usr-****");
        assert_eq!(mask_user_id("ab"), "usr-**");
        // longer IDs show last 4 chars
        assert_eq!(mask_user_id("user-x-0001"), "usr-***-0001");
    }
}
