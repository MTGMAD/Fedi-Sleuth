// ============================================================================
// Platform Trait - Unified interface for all social platforms
// ============================================================================
// This trait defines the common interface that all platform services must
// implement. Each platform (Pixelfed, Mastodon, Bluesky) has its own
// implementation file with platform-specific API calls.
// ============================================================================

use anyhow::Result;
use async_trait::async_trait;

use crate::models::{Platform, SearchResult, SearchType};

/// Common interface for all social media platforms
#[async_trait]
pub trait SocialPlatform: Send + Sync {
    /// Get the platform identifier
    fn platform(&self) -> Platform;

    /// Check if the platform is currently authenticated
    fn is_authenticated(&self) -> bool;

    /// Check if the platform is enabled in settings
    fn is_enabled(&self) -> bool;

    /// Get the platform's instance URL (or base URL for Bluesky)
    fn instance_url(&self) -> &str;

    /// Search for posts from a specific user
    ///
    /// # Arguments
    /// * `username` - The username to search for (e.g., "@user@instance.social")
    /// * `days_back` - Number of days to search back
    ///
    /// # Returns
    /// Vector of SearchResult with platform field populated
    async fn search_user(&self, username: &str, days_back: u32) -> Result<Vec<SearchResult>>;

    /// Search for posts by hashtag
    ///
    /// # Arguments
    /// * `hashtag` - The hashtag to search for (e.g., "#photography")
    /// * `days_back` - Number of days to search back
    ///
    /// # Returns
    /// Vector of SearchResult with platform field populated
    async fn search_hashtag(&self, hashtag: &str, days_back: u32) -> Result<Vec<SearchResult>>;

    /// Generic search method that dispatches to user or hashtag search
    async fn search(
        &self,
        query: String,
        search_type: SearchType,
        days_back: u32,
    ) -> Result<Vec<SearchResult>> {
        match search_type {
            SearchType::User => self.search_user(&query, days_back).await,
            SearchType::Hashtag => self.search_hashtag(&query, days_back).await,
        }
    }
}

/// Helper to create a descriptive name for logging
pub fn platform_display_name(platform: Platform, instance_url: &str) -> String {
    match platform {
        Platform::Pixelfed => format!("{} Pixelfed ({})", platform.emoji(), instance_url),
        Platform::Mastodon => format!("{} Mastodon ({})", platform.emoji(), instance_url),
        Platform::Bluesky => format!("{} Bluesky", platform.emoji()),
    }
}
