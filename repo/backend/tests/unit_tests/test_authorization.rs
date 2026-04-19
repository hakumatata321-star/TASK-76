use fleetreserve_backend::handlers::auth::{enforce_store_isolation, require_role};
use fleetreserve_backend::models::{Claims, UserRole};

fn claims(role: &str, store_id: Option<&str>) -> Claims {
    Claims {
        user_id: "u1".into(),
        username: "alice".into(),
        role: role.into(),
        store_id: store_id.map(|s| s.to_string()),
        iat: 0,
        exp: i64::MAX,
    }
}

#[test]
fn unit_authorization_role_hierarchy() {
    assert!(require_role(&claims("Administrator", None), &UserRole::Customer).is_ok());
    assert!(require_role(&claims("Customer", None), &UserRole::MerchantStaff).is_err());
}

#[test]
fn unit_authorization_store_isolation() {
    let merchant = claims("MerchantStaff", Some("store-001"));
    assert!(enforce_store_isolation(&merchant, "store-001").is_ok());
    assert!(enforce_store_isolation(&merchant, "store-002").is_err());
}
