/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub mongo_uri: String,
    pub mongo_db: String,
    pub jwt_secret: String,
    pub jwt_expiry_seconds: u64,
    pub server_port: String,
}

impl AppConfig {
    /// Load configuration from environment variables (via `dotenvy`).
    ///
    /// # Panics
    /// Panics if `JWT_SECRET` is absent.
    pub fn from_env() -> Self {
        Self {
            mongo_uri: std::env::var("MONGO_URI")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            mongo_db: std::env::var("MONGO_DB").unwrap_or_else(|_| "blog".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change_me_in_production".to_string()),
            jwt_expiry_seconds: std::env::var("JWT_EXPIRY_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "0.0.0.0:3001".to_string()),
        }
    }
}
