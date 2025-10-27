use crate::models::AppSettings;
use anyhow::Result;

pub struct SettingsService;

impl SettingsService {
    pub async fn load_settings() -> Result<AppSettings> {
        match confy::load("pixelfed-rust", "settings") {
            Ok(settings) => Ok(settings),
            Err(_) => {
                // If loading fails, return default settings and save them
                let default_settings = AppSettings::default();
                let _ = Self::save_settings(&default_settings).await;
                Ok(default_settings)
            }
        }
    }

    pub async fn save_settings(settings: &AppSettings) -> Result<()> {
        confy::store("pixelfed-rust", "settings", settings)
            .map_err(|e| anyhow::anyhow!("Failed to save settings: {}", e))
    }
}
