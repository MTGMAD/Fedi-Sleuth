// ============================================================================
// Bluesky Service - ATProto API implementation
// ============================================================================
// Provides search capabilities against the Bluesky social network using the
// SocialPlatform trait interface. Authenticates with app passwords and calls
// the official ATProto endpoints (createSession, getAuthorFeed, searchPosts).
// ============================================================================

use std::time::Duration as StdDuration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::{Client, Response};
use serde::Deserialize;
use serde_json::Value;
use tokio::time::{sleep, Duration as TokioDuration};

use crate::models::{AppSettings, BlueskyAuth, Platform, SearchResult};
use crate::services::SocialPlatform;

const BLUESKY_API_BASE: &str = "https://bsky.social";
const BLUESKY_WEB_BASE: &str = "https://bsky.app";

pub struct BlueskyService {
    client: Client,
    auth: BlueskyAuth,
}

impl BlueskyService {
    pub fn new(settings: &AppSettings) -> Self {
        let client = Client::builder()
            .user_agent("Fedi-Sleuth/0.1.0")
            .timeout(StdDuration::from_secs(45))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            auth: settings.api.bluesky.clone(),
        }
    }

    fn ensure_enabled(&self) -> Result<()> {
        if !self.auth.enabled {
            return Err(anyhow::anyhow!(
                "Bluesky is disabled. Enable it in Settings and provide credentials."
            ));
        }

        if self.auth.handle.trim().is_empty() || self.auth.app_password.trim().is_empty() {
            return Err(anyhow::anyhow!(
                "Bluesky handle or app password missing. Update Settings with valid credentials."
            ));
        }

        Ok(())
    }

    async fn create_session(&self) -> Result<BlueskySession> {
        self.ensure_enabled()?;

        let url = format!("{}/xrpc/com.atproto.server.createSession", BLUESKY_API_BASE);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "identifier": self.auth.handle.trim(),
                "password": self.auth.app_password.trim()
            }))
            .send()
            .await
            .with_context(|| "Failed to contact Bluesky session endpoint")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Bluesky login failed: {}. Response: {}",
                status,
                body
            ));
        }

        let session: CreateSessionResponse = response
            .json()
            .await
            .with_context(|| "Failed to decode Bluesky session response")?;

        Ok(BlueskySession {
            access_jwt: session.access_jwt,
        })
    }

    async fn api_get(
        &self,
        session: &BlueskySession,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Response> {
        let mut request = self
            .client
            .get(format!("{}{}", BLUESKY_API_BASE, path))
            .bearer_auth(&session.access_jwt);

        let params: Vec<(&str, &str)> = query
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect();

        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("Failed to fetch Bluesky endpoint {}", path))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Bluesky request failed: {}. Response: {}",
                status,
                body
            ));
        }

        Ok(response)
    }

    async fn search_user_posts_internal(
        &self,
        handle: &str,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>> {
        let session = self.create_session().await?;
        let mut results = Vec::new();
        let mut cursor: Option<String> = None;
        let mut pages = 0u32;

        loop {
            pages += 1;
            if pages > 120 {
                log::warn!("Bluesky author feed aborted after {} pages", pages);
                break;
            }

            let mut query = vec![("actor", handle.to_string()), ("limit", "30".to_string())];

            if let Some(ref value) = cursor {
                query.push(("cursor", value.clone()));
            }

            let response = self
                .api_get(&session, "/xrpc/app.bsky.feed.getAuthorFeed", &query)
                .await?;

            let response: BlueskyFeedResponse = response
                .json()
                .await
                .with_context(|| "Failed to decode Bluesky author feed")?;

            let BlueskyFeedResponse {
                feed,
                cursor: next_cursor,
            } = response;

            if feed.is_empty() {
                break;
            }

            let mut processed_any = false;
            let mut found_old_post = false;

            for item in feed {
                let post = item.post;

                if let Some(result) = Self::convert_post(&post, cutoff_date) {
                    processed_any = true;
                    results.push(result);
                } else if let Some(created_at) = Self::parse_created_at(&post) {
                    if created_at < cutoff_date {
                        found_old_post = true;
                    }
                }
            }

            if found_old_post {
                break;
            }

            if let Some(next_cursor) = next_cursor {
                if cursor
                    .as_ref()
                    .map(|value| value == &next_cursor)
                    .unwrap_or(false)
                {
                    break;
                }
                cursor = Some(next_cursor);
            } else {
                break;
            }

            if !processed_any {
                break;
            }

            sleep(TokioDuration::from_millis(100)).await;
        }

        Ok(results)
    }

    async fn search_hashtag_posts_internal(
        &self,
        hashtag: &str,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>> {
        let session = self.create_session().await?;
        let mut results = Vec::new();
        let mut cursor: Option<String> = None;
        let mut pages = 0u32;
        let query_string = format!("#{hashtag}");

        loop {
            pages += 1;
            if pages > 120 {
                log::warn!("Bluesky search aborted after {} pages", pages);
                break;
            }

            let mut query = vec![("q", query_string.clone()), ("limit", "30".to_string())];

            if let Some(ref value) = cursor {
                query.push(("cursor", value.clone()));
            }

            let response = self
                .api_get(&session, "/xrpc/app.bsky.feed.searchPosts", &query)
                .await?;

            let response: BlueskySearchResponse = response
                .json()
                .await
                .with_context(|| "Failed to decode Bluesky hashtag search")?;

            let BlueskySearchResponse {
                posts,
                cursor: next_cursor,
            } = response;

            if posts.is_empty() {
                break;
            }

            let mut processed_any = false;
            let mut found_old_post = false;

            for post in posts {
                if let Some(result) = Self::convert_post(&post, cutoff_date) {
                    processed_any = true;
                    results.push(result);
                } else if let Some(created_at) = Self::parse_created_at(&post) {
                    if created_at < cutoff_date {
                        found_old_post = true;
                    }
                }
            }

            if found_old_post {
                break;
            }

            if let Some(next_cursor) = next_cursor {
                if cursor
                    .as_ref()
                    .map(|value| value == &next_cursor)
                    .unwrap_or(false)
                {
                    break;
                }
                cursor = Some(next_cursor);
            } else {
                break;
            }

            if !processed_any {
                break;
            }

            sleep(TokioDuration::from_millis(100)).await;
        }

        Ok(results)
    }

    fn convert_post(post: &BlueskyPostView, cutoff_date: DateTime<Utc>) -> Option<SearchResult> {
        let created_at = Self::parse_created_at(post)?;
        if created_at < cutoff_date {
            return None;
        }

        let author = post
            .author
            .display_name
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or(&post.author.handle);

        let (media_urls, media_types, media_count) = Self::extract_media(post.embed.as_ref());

        Some(SearchResult {
            platform: Platform::Bluesky,
            id: post.uri.clone(),
            author: author.to_string(),
            content: post.record.text.as_deref().unwrap_or("").trim().to_string(),
            created_at,
            media_urls,
            media_types,
            media_count,
            likes: post.like_count.unwrap_or(0),
            shares: post.repost_count.unwrap_or(0),
            url: Self::web_url(&post.author.handle, &post.uri),
        })
    }

    fn parse_created_at(post: &BlueskyPostView) -> Option<DateTime<Utc>> {
        let source = post
            .record
            .created_at
            .as_deref()
            .or_else(|| post.indexed_at.as_deref())?;

        DateTime::parse_from_rfc3339(source)
            .map(|value| value.with_timezone(&Utc))
            .ok()
    }

    fn extract_media(embed: Option<&Value>) -> (Vec<String>, Vec<String>, u32) {
        let mut urls = Vec::new();
        let mut types = Vec::new();

        if let Some(value) = embed {
            Self::extract_media_recursive(value, &mut urls, &mut types);
        }

        let count = urls.len() as u32;
        (urls, types, count)
    }

    fn extract_media_recursive(value: &Value, urls: &mut Vec<String>, types: &mut Vec<String>) {
        if let Some(object) = value.as_object() {
            match object.get("$type").and_then(Value::as_str) {
                Some("app.bsky.embed.images#view") => {
                    if let Some(images) = object.get("images").and_then(Value::as_array) {
                        for image in images {
                            if let Some(fullsize) = image.get("fullsize").and_then(Value::as_str) {
                                let trimmed = fullsize.trim();
                                if !trimmed.is_empty() {
                                    urls.push(trimmed.to_string());
                                    types.push("image".to_string());
                                }
                            }
                        }
                    }
                }
                Some("app.bsky.embed.external#view") => {
                    if let Some(external) = object.get("external") {
                        if let Some(uri) = external.get("uri").and_then(Value::as_str) {
                            let trimmed = uri.trim();
                            if !trimmed.is_empty() {
                                urls.push(trimmed.to_string());
                                types.push("external".to_string());
                            }
                        }
                    }
                }
                Some("app.bsky.embed.video#view") => {
                    if let Some(playlist) = object.get("playlist").and_then(Value::as_str) {
                        let trimmed = playlist.trim();
                        if !trimmed.is_empty() {
                            urls.push(trimmed.to_string());
                            types.push("video".to_string());
                        }
                    }
                }
                Some("app.bsky.embed.recordWithMedia#view") => {
                    if let Some(media) = object.get("media") {
                        Self::extract_media_recursive(media, urls, types);
                    }
                }
                _ => {}
            }
        }
    }

    fn web_url(handle: &str, uri: &str) -> String {
        let rkey = uri.rsplit('/').next().unwrap_or("post");
        format!("{}/profile/{}/post/{}", BLUESKY_WEB_BASE, handle, rkey)
    }
}

#[async_trait]
impl SocialPlatform for BlueskyService {
    fn platform(&self) -> Platform {
        Platform::Bluesky
    }

    fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
            || (self.auth.enabled
                && !self.auth.handle.trim().is_empty()
                && !self.auth.app_password.trim().is_empty())
    }

    fn is_enabled(&self) -> bool {
        self.auth.enabled
    }

    fn instance_url(&self) -> &str {
        BLUESKY_API_BASE
    }

    async fn search_user(&self, username: &str, days_back: u32) -> Result<Vec<SearchResult>> {
        let cutoff_date = Utc::now() - Duration::days(days_back as i64);
        let cleaned = username.trim().trim_start_matches('@');
        self.search_user_posts_internal(cleaned, cutoff_date).await
    }

    async fn search_hashtag(&self, hashtag: &str, days_back: u32) -> Result<Vec<SearchResult>> {
        let cutoff_date = Utc::now() - Duration::days(days_back as i64);
        let cleaned = hashtag.trim().trim_start_matches('#');
        self.search_hashtag_posts_internal(cleaned, cutoff_date)
            .await
    }
}

#[derive(Debug, Deserialize)]
struct CreateSessionResponse {
    #[serde(rename = "accessJwt")]
    access_jwt: String,
}

struct BlueskySession {
    access_jwt: String,
}

#[derive(Debug, Deserialize)]
struct BlueskyFeedResponse {
    #[serde(default)]
    feed: Vec<BlueskyFeedItem>,
    #[serde(default)]
    cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlueskyFeedItem {
    post: BlueskyPostView,
}

#[derive(Debug, Deserialize)]
struct BlueskySearchResponse {
    #[serde(default)]
    posts: Vec<BlueskyPostView>,
    #[serde(default)]
    cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlueskyPostView {
    uri: String,
    author: BlueskyProfileView,
    record: BlueskyRecord,
    #[serde(default)]
    embed: Option<Value>,
    #[serde(rename = "likeCount")]
    #[serde(default)]
    like_count: Option<u32>,
    #[serde(rename = "repostCount")]
    #[serde(default)]
    repost_count: Option<u32>,
    #[serde(rename = "indexedAt")]
    #[serde(default)]
    indexed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlueskyProfileView {
    handle: String,
    #[serde(rename = "displayName")]
    #[serde(default)]
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlueskyRecord {
    #[serde(default)]
    text: Option<String>,
    #[serde(rename = "createdAt")]
    #[serde(default)]
    created_at: Option<String>,
}
