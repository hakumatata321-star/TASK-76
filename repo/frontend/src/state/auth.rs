use leptos::*;

#[derive(Clone, Debug)]
pub struct AuthState {
    pub token: RwSignal<Option<String>>,
    pub csrf_token: RwSignal<Option<String>>,
    pub user_id: RwSignal<Option<String>>,
    pub username: RwSignal<Option<String>>,
    pub display_name: RwSignal<Option<String>>,
    pub role: RwSignal<Option<String>>,
    pub store_id: RwSignal<Option<String>>,
    pub is_authenticated: RwSignal<bool>,
}

impl AuthState {
    pub fn new() -> Self {
        Self {
            token: create_rw_signal(None),
            csrf_token: create_rw_signal(None),
            user_id: create_rw_signal(None),
            username: create_rw_signal(None),
            display_name: create_rw_signal(None),
            role: create_rw_signal(None),
            store_id: create_rw_signal(None),
            is_authenticated: create_rw_signal(false),
        }
    }

    pub fn login(&self, token: String, csrf: String, user_id: String, username: String, display_name: String, role: String, store_id: Option<String>) {
        self.token.set(Some(token));
        self.csrf_token.set(Some(csrf));
        self.user_id.set(Some(user_id));
        self.username.set(Some(username));
        self.display_name.set(Some(display_name));
        self.role.set(Some(role));
        self.store_id.set(store_id);
        self.is_authenticated.set(true);
    }

    pub fn logout(&self) {
        self.token.set(None);
        self.csrf_token.set(None);
        self.user_id.set(None);
        self.username.set(None);
        self.display_name.set(None);
        self.role.set(None);
        self.store_id.set(None);
        self.is_authenticated.set(false);
    }

    pub fn update_token(&self, token: String) {
        self.token.set(Some(token));
    }

    pub fn current_role(&self) -> Option<String> {
        self.role.get()
    }

    pub fn has_role(&self, required: &str) -> bool {
        satisfies_role(self.role.get().as_deref(), required)
    }
}

/// Numeric rank for role-based access (higher = more privilege).
pub fn role_rank(role: &str) -> u8 {
    match role {
        "Customer" => 1,
        "Photographer" => 2,
        "MerchantStaff" => 3,
        "PlatformOps" => 4,
        "Administrator" => 5,
        _ => 0,
    }
}

pub fn satisfies_role(current: Option<&str>, required: &str) -> bool {
    match current {
        Some(r) => role_rank(r) >= role_rank(required),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_rank_ordering() {
        assert!(role_rank("Administrator") > role_rank("PlatformOps"));
        assert!(role_rank("PlatformOps") > role_rank("MerchantStaff"));
        assert!(role_rank("MerchantStaff") > role_rank("Photographer"));
        assert!(role_rank("Photographer") > role_rank("Customer"));
    }

    #[test]
    fn satisfies_role_hierarchy() {
        assert!(satisfies_role(Some("Administrator"), "Customer"));
        assert!(satisfies_role(Some("MerchantStaff"), "MerchantStaff"));
        assert!(!satisfies_role(Some("Customer"), "MerchantStaff"));
        assert!(!satisfies_role(None, "Customer"));
    }

    #[test]
    fn unknown_role_is_lowest() {
        assert_eq!(role_rank("Nope"), 0);
        assert!(!satisfies_role(Some("Nope"), "Customer"));
    }
}
