use std::env;

pub struct AppConfig {
    pub gemini_api_key: String,
}

impl AppConfig {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();

        let gemini_api_key = env::var("GEMINI_API_KEY")
            .expect("GEMINI_API_KEY must be set in environment");

        AppConfig { gemini_api_key }
    }
}
