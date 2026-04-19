use crate::state::auth::AuthState;
use leptos::*;

pub fn get_csrf_token() -> Option<String> {
    let auth = use_context::<AuthState>()?;
    auth.csrf_token.get()
}
