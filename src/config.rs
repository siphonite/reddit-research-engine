use std::env;

pub struct AppConfig {
    pub gemini_api_key: String,
    pub google_sheet_id: Option<String>,
    pub google_credentials_path: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();

        let gemini_api_key = env::var("GEMINI_API_KEY")
            .expect("GEMINI_API_KEY must be set in environment");

        let google_sheet_id = env::var("GOOGLE_SHEET_ID").ok();
        let google_credentials_path = env::var("GOOGLE_APPLICATION_CREDENTIALS").ok();

        AppConfig {
            gemini_api_key,
            google_sheet_id,
            google_credentials_path,
        }
    }

    /// Returns true if both Sheet ID and credentials are configured.
    pub fn sheets_enabled(&self) -> bool {
        self.google_sheet_id.is_some() && self.google_credentials_path.is_some()
    }
}
