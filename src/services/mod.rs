// ============================================================================
// Services Module - Business logic and API integrations
// ============================================================================
// Each platform has its own service file that implements the SocialPlatform trait

pub mod auth_service;
pub mod bluesky_service;
pub mod download_service;
pub mod mastodon_service;
pub mod pixelfed_service;
pub mod platform_trait;
pub mod settings_service;

pub use auth_service::AuthService;
pub use bluesky_service::BlueskyService;
pub use download_service::DownloadService;
pub use mastodon_service::MastodonService;
pub use pixelfed_service::PixelfedService;
pub use platform_trait::{platform_display_name, SocialPlatform};
pub use settings_service::SettingsService;
