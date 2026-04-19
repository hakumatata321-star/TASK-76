#[path = "../src/state/auth.rs"]
mod auth_state;

#[test]
fn frontend_auth_role_behavior_is_enforced() {
    assert!(auth_state::satisfies_role(Some("Administrator"), "PlatformOps"));
    assert!(auth_state::satisfies_role(Some("MerchantStaff"), "Customer"));
    assert!(!auth_state::satisfies_role(Some("Customer"), "MerchantStaff"));
    assert!(!auth_state::satisfies_role(None, "Customer"));
}

#[test]
fn frontend_auth_role_rank_order_is_stable() {
    assert!(auth_state::role_rank("Administrator") > auth_state::role_rank("PlatformOps"));
    assert!(auth_state::role_rank("PlatformOps") > auth_state::role_rank("MerchantStaff"));
    assert!(auth_state::role_rank("MerchantStaff") > auth_state::role_rank("Photographer"));
    assert!(auth_state::role_rank("Photographer") > auth_state::role_rank("Customer"));
}
