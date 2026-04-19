use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub encryption_key: String,
    pub hmac_secret: String,
    pub upload_dir: String,
    /// Server-side CSRF token store: maps user_id -> active CSRF token
    pub csrf_tokens: Arc<Mutex<HashMap<String, String>>>,
    /// Revoked session keys: "user_id:iat" tuples for tokens explicitly invalidated via logout.
    /// Checked on every authenticated request to enforce immediate logout.
    pub revoked_sessions: Arc<Mutex<HashSet<String>>>,
}
