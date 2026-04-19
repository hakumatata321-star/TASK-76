#[derive(Clone, Debug)]
pub struct AppConfig {
    pub api_base: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_base: "/api".to_string(),
        }
    }
}
