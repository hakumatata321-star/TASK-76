use std::sync::{Arc, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use fleetreserve_backend::app::state::AppState;
use fleetreserve_backend::routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting FleetReserve Operations Suite backend");

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "fleetreserve.db".to_string());
    let conn = rusqlite::Connection::open(&db_url).expect("Failed to open database");
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .expect("Failed to set pragmas");

    run_migrations(&conn);
    bootstrap_admin(&conn);

    let encryption_key = std::env::var("ENCRYPTION_KEY")
        .expect("ENCRYPTION_KEY env var is required — see .env.example");
    let hmac_secret = std::env::var("HMAC_SECRET")
        .expect("HMAC_SECRET env var is required — see .env.example");
    if encryption_key.len() < 32 {
        panic!("ENCRYPTION_KEY must be at least 32 characters");
    }
    if hmac_secret.len() < 32 {
        panic!("HMAC_SECRET must be at least 32 characters");
    }
    // Refuse to start with the well-known placeholder values committed in docker-compose.yml.
    // Operators must supply unique secrets via .env or docker secrets before deploying.
    if encryption_key == "change-this-32-byte-key-in-prod!"
        || hmac_secret == "change-this-hmac-secret-in-prod!"
    {
        panic!(
            "ENCRYPTION_KEY and HMAC_SECRET must be changed from their placeholder values. \
             Copy .env.example to .env, fill in unique secrets, then run: \
             docker-compose --env-file .env up"
        );
    }
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());

    std::fs::create_dir_all(&upload_dir).expect("Failed to create upload directory");

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        encryption_key,
        hmac_secret,
        upload_dir,
        csrf_tokens: Arc::new(Mutex::new(std::collections::HashMap::new())),
        revoked_sessions: Arc::new(Mutex::new(std::collections::HashSet::new())),
    };

    let app = routes::build_router(state);

    let addr = "0.0.0.0:3001";
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn run_migrations(conn: &rusqlite::Connection) {
    let schema = include_str!("../migrations/001_initial_schema.sql");
    conn.execute_batch(schema).expect("Failed to run schema migration");

    let seed = include_str!("../migrations/002_seed_data.sql");
    conn.execute_batch(seed).expect("Failed to run seed migration");

    tracing::info!("Database migrations complete");
}

/// On first run the seed leaves admin disabled. Activate it with the well-known
/// default credentials so `docker-compose up` works out of the box on a clean
/// install. Set `BOOTSTRAP_ADMIN_PASSWORD` in .env to override the default.
fn bootstrap_admin(conn: &rusqlite::Connection) {
    let is_inactive: bool = conn
        .query_row(
            "SELECT active FROM users WHERE id = 'user-admin-001'",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map(|active| active == 0)
        .unwrap_or(false);

    if is_inactive {
        let password = std::env::var("BOOTSTRAP_ADMIN_PASSWORD")
            .unwrap_or_else(|_| "FleetReserveHttpTest#2026".to_string());
        let hash = fleetreserve_backend::auth::password::hash_password(&password)
            .expect("Failed to hash bootstrap admin password");
        conn.execute(
            "UPDATE users SET active = 1, password_hash = ?1 WHERE id = 'user-admin-001' AND active = 0",
            [&hash],
        )
        .expect("Failed to activate bootstrap admin");
        tracing::info!(
            "Bootstrap: admin activated with default credentials (username=admin). \
             Change password on first login via the admin UI or recovery-code flow."
        );
    }
}
