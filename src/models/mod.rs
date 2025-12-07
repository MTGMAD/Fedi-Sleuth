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

impl AppSettings {
    /// Migrate from old single-platform settings to multi-platform
    pub fn migrate_from_legacy(instance_url: String, old_api: LegacyApiSettings) -> Self {
        Self {
            instance_url: instance_url.clone(),
            appearance: AppearanceSettings::default(),
            api: ApiSettings {
                pixelfed: PlatformAuth {
                    enabled: old_api.use_oauth,
                    instance_url,
                    app_name: old_api.app_name,
                    client_id: old_api.client_id,
                    client_secret: old_api.client_secret,
                    access_token: old_api.access_token,
                },
                mastodon: PlatformAuth {
                    enabled: false,
                    instance_url: "mastodon.social".to_string(),
                    app_name: "Fedi Sleuth".to_string(),
                    client_id: String::new(),
                    client_secret: String::new(),
                    access_token: None,
                },
                bluesky: BlueskyAuth::default(),
            },
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

// ============================================================================
// Multi-Platform API Settings
// ============================================================================

/// Legacy API settings structure for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyApiSettings {
    pub use_oauth: bool,
    pub app_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSettings {
    pub pixelfed: PlatformAuth,
    pub mastodon: PlatformAuth,
    pub bluesky: BlueskyAuth,
}

impl Default for ApiSettings {
    fn default() -> Self {
        Self {
            pixelfed: PlatformAuth {
                enabled: false,
                instance_url: "pixelfed.social".to_string(),
                app_name: "Fedi Sleuth".to_string(),
                client_id: String::new(),
                client_secret: String::new(),
                access_token: None,
            },
            mastodon: PlatformAuth {
                enabled: false,
                instance_url: "mastodon.social".to_string(),
                app_name: "Fedi Sleuth".to_string(),
                client_id: String::new(),
                client_secret: String::new(),
                access_token: None,
            },
            bluesky: BlueskyAuth::default(),
        }
    }
}

/// OAuth 2.0 authentication for Pixelfed and Mastodon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAuth {
    pub enabled: bool,
    pub instance_url: String,
    pub app_name: String,
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
}

impl PlatformAuth {
    pub fn is_authenticated(&self) -> bool {
        self.enabled
            && self.access_token.is_some()
            && !self.access_token.as_ref().unwrap().is_empty()
    }
}

/// ATProto authentication for Bluesky
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueskyAuth {
    pub enabled: bool,
    pub handle: String,
    pub app_password: String,
    pub did: Option<String>,
    pub access_jwt: Option<String>,
    pub refresh_jwt: Option<String>,
}

impl Default for BlueskyAuth {
    fn default() -> Self {
        Self {
            enabled: false,
            handle: String::new(),
            app_password: String::new(),
            did: None,
            access_jwt: None,
            refresh_jwt: None,
        }
    }
}

impl BlueskyAuth {
    pub fn is_authenticated(&self) -> bool {
        self.enabled && self.access_jwt.is_some() && !self.access_jwt.as_ref().unwrap().is_empty()
    }
}

/// Platform identifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Platform {
    Pixelfed,
    Mastodon,
    Bluesky,
}

impl Platform {
    pub fn name(&self) -> &str {
        match self {
            Platform::Pixelfed => "Pixelfed",
            Platform::Mastodon => "Mastodon",
            Platform::Bluesky => "Bluesky",
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            Platform::Pixelfed => "🟣",
            Platform::Mastodon => "🐘",
            Platform::Bluesky => "🦋",
        }
    }

    pub fn folder_name(&self) -> &str {
        match self {
            Platform::Pixelfed => "Pixelfed",
            Platform::Mastodon => "Mastodon",
            Platform::Bluesky => "bsky",
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

impl SearchType {
    pub fn get_folder_prefix(&self, query: &str) -> String {
        match self {
            SearchType::User => {
                // Remove @ symbols and clean username
                query.trim_start_matches('@').replace('@', "_at_")
            }
            SearchType::Hashtag => {
                // Remove # symbol
                query.trim_start_matches('#').to_string()
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub platform: Platform,
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
    pub content: Option<String>,
    pub created_at: Option<String>,
    #[serde(default)]
    pub media_attachments: Vec<PixelfedMediaAttachment>,
    #[serde(default)]
    pub favourites_count: Option<u32>,
    #[serde(default)]
    pub reblogs_count: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelfedAccount {
    pub id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelfedMediaAttachment {
    pub id: String,
    pub r#type: Option<String>,
    pub url: Option<String>,
    pub preview_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DownloadProgress {
    pub current: usize,
    pub total: usize,
    pub current_file: String,
}

// ============================================================================
// Multi-Platform Search Context
// ============================================================================

/// Context for the current search, including query and type
#[derive(Debug, Clone)]
pub struct SearchContext {
    pub query: String,
    pub search_type: SearchType,
    pub days_back: u32,
}

impl SearchContext {
    pub fn new(query: String, search_type: SearchType, days_back: u32) -> Self {
        Self {
            query,
            search_type,
            days_back,
        }
    }

    pub fn get_folder_name(&self) -> String {
        self.search_type.get_folder_prefix(&self.query)
    }
}

/// Results grouped by platform
#[derive(Debug, Clone)]
pub struct PlatformSearchResults {
    pub platform: Platform,
    pub label: String,
    pub results: Vec<SearchResult>,
    pub error: Option<String>,
}

impl PlatformSearchResults {
    pub fn success(platform: Platform, label: String, results: Vec<SearchResult>) -> Self {
        Self {
            platform,
            label,
            results,
            error: None,
        }
    }

    pub fn error(platform: Platform, label: String, error: String) -> Self {
        Self {
            platform,
            label,
            results: Vec::new(),
            error: Some(error),
        }
    }

    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    pub fn count(&self) -> usize {
        self.results.len()
    }
}
