use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AppState {
    pub settings: AppSettings,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: AppSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub instance_url: String,
    pub appearance: AppearanceSettings,
    pub api: ApiSettings,
    pub download: DownloadSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            instance_url: "pixelfed.social".to_string(),
            appearance: AppearanceSettings::default(),
            api: ApiSettings::default(),
            download: DownloadSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme: Theme,
    pub accent_color: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            accent_color: "#0078d4".to_string(), // Windows 11 default blue
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light => write!(f, "light"),
            Theme::Dark => write!(f, "dark"),
            Theme::System => write!(f, "system"),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiSettings {
    pub use_oauth: bool,
    pub app_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
}

impl Default for ApiSettings {
    fn default() -> Self {
        Self {
            use_oauth: false,
            app_name: "Pixelfed Rust Client".to_string(),
            client_id: String::new(),
            client_secret: String::new(),
            access_token: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSettings {
    pub base_path: String,
    pub max_concurrent: u32,
    pub organize_by_date: bool,
}

impl Default for DownloadSettings {
    fn default() -> Self {
        let downloads_dir = dirs::download_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "./downloads".to_string());

        Self {
            base_path: downloads_dir,
            max_concurrent: 3,
            organize_by_date: true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SearchType {
    User,
    Hashtag,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub author: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub media_urls: Vec<String>,
    pub media_types: Vec<String>,
    pub media_count: u32,
    pub likes: u32,
    pub shares: u32,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelfedPost {
    pub id: String,
    pub account: PixelfedAccount,
    pub content: String,
    pub created_at: String,
    pub media_attachments: Vec<PixelfedMediaAttachment>,
    pub favourites_count: u32,
    pub reblogs_count: u32,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelfedAccount {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelfedMediaAttachment {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub preview_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DownloadProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}
