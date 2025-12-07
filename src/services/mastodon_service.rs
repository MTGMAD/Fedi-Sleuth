// ============================================================================
// Mastodon Service - Mastodon API implementation
// ============================================================================
// Mirrors the SocialPlatform trait using Mastodon's REST API.
// ============================================================================

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;

use crate::models::{AppSettings, PixelfedPost, Platform, SearchResult};
use crate::services::SocialPlatform;

pub struct MastodonService {
    client: Client,
    instance_url: String,
    access_token: Option<String>,
    enabled: bool,
}

impl MastodonService {
    pub fn new(settings: &AppSettings) -> Self {
        let platform_auth = &settings.api.mastodon;
        let trimmed = platform_auth.instance_url.trim();

        let normalized_url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            trimmed.trim_end_matches('/').to_string()
        } else {
            format!("https://{}", trimmed.trim_end_matches('/'))
        };

        let client = Client::builder()
            .user_agent("Fedi-Sleuth/0.1.0")
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            instance_url: normalized_url,
            access_token: platform_auth.access_token.clone(),
            enabled: platform_auth.enabled,
        }
    }

    fn require_access_token(&self) -> Result<&str> {
        self.access_token
            .as_deref()
            .filter(|token| !token.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Authentication required. Please enable OAuth in Settings and sign in."
                )
            })
    }

    fn fallback_post_url(&self, post: &PixelfedPost) -> String {
        let base = self.instance_url.trim_end_matches('/');
        let username = post
            .account
            .username
            .as_deref()
            .map(|value| value.trim_start_matches('@'))
            .filter(|value| !value.is_empty())
            .unwrap_or("unknown");

        format!("{}/@{}/{}", base, username, post.id)
    }

    fn account_name(post: &PixelfedPost) -> String {
        post.account
            .display_name
            .as_deref()
            .filter(|value| !value.is_empty())
            .or_else(|| {
                post.account
                    .username
                    .as_deref()
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or("Unknown")
            .to_string()
    }

    fn extract_media(post: &PixelfedPost) -> (Vec<String>, Vec<String>, u32) {
        let mut urls = Vec::new();
        let mut types = Vec::new();

        for attachment in &post.media_attachments {
            if let Some(url) = attachment.url.as_deref() {
                let trimmed = url.trim();
                if trimmed.is_empty() {
                    continue;
                }
                urls.push(trimmed.to_string());
                types.push(attachment.r#type.as_deref().unwrap_or("").to_string());
            }
        }

        let count = urls.len() as u32;
        (urls, types, count)
    }

    async fn search_user_posts(
        &self,
        username: &str,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>> {
        let access_token = self.require_access_token()?;

        let clean_username = username.trim_start_matches('@');
        let search_query = clean_username.to_string();

        let search_url = format!(
            "{}/api/v2/search?q={}&type=accounts&resolve=true&limit=1",
            self.instance_url,
            urlencoding::encode(&search_query)
        );

        log::info!(
            "Searching for Mastodon user '{}' via {}",
            search_query,
            search_url
        );

        let response = self
            .client
            .get(&search_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .timeout(std::time::Duration::from_secs(45))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "User search failed: {}. Response: {}. User '{}' may not exist or is unreachable.",
                status,
                body,
                search_query
            ));
        }

        let data: serde_json::Value = response.json().await?;
        let accounts = data["accounts"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid search response"))?;

        if accounts.is_empty() {
            return Err(anyhow::anyhow!(
                "User '{}' not found on {}. Try searching directly on their home instance.",
                search_query,
                self.instance_url
            ));
        }

        let user_id = accounts[0]["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid user data"))?;

        let timeline_url = format!(
            "{}/api/v1/accounts/{}/statuses?limit=40",
            self.instance_url, user_id
        );
        self.fetch_timeline(&timeline_url, cutoff_date, Some(access_token))
            .await
    }

    async fn search_hashtag_posts(
        &self,
        hashtag: &str,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>> {
        let access_token = self.require_access_token()?;
        let clean_hashtag = hashtag.trim_start_matches('#');

        let timeline_url = format!(
            "{}/api/v1/timelines/tag/{}?limit=40",
            self.instance_url, clean_hashtag
        );

        self.fetch_timeline(&timeline_url, cutoff_date, Some(access_token))
            .await
    }

    async fn fetch_timeline(
        &self,
        base_url: &str,
        cutoff_date: DateTime<Utc>,
        access_token: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let mut max_id: Option<String> = None;
        let mut page = 0u32;

        loop {
            page += 1;
            if page > 120 {
                log::warn!("Mastodon timeline fetch aborted after {} pages", page);
                break;
            }

            let mut url = base_url.to_string();
            if let Some(ref id) = max_id {
                let join_char = if url.contains('?') { '&' } else { '?' };
                url.push_str(&format!("{}max_id={}", join_char, id));
            }

            log::info!("Fetching Mastodon timeline page {}: {}", page, url);

            let mut request = self.client.get(&url);
            if let Some(token) = access_token {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request.send().await?;
            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "Failed to fetch timeline: {}. Response: {}",
                    status,
                    body
                ));
            }

            let posts: Vec<PixelfedPost> = response.json().await?;
            if posts.is_empty() {
                break;
            }

            let mut found_old_post = false;
            let mut processed_any = false;
            let mut fallback_next_max_id: Option<String> = None;

            for post in posts {
                if post.id.is_empty() {
                    continue;
                }

                let post_id = post.id.clone();
                fallback_next_max_id = Some(post_id.clone());

                let created_at_str = match post.created_at.as_deref() {
                    Some(value) if !value.is_empty() => value,
                    _ => continue,
                };

                let created_at = match DateTime::parse_from_rfc3339(created_at_str) {
                    Ok(dt) => dt.with_timezone(&Utc),
                    Err(_) => continue,
                };

                if created_at < cutoff_date {
                    found_old_post = true;
                    break;
                }

                let author = Self::account_name(&post);
                let (media_urls, media_types, media_count) = Self::extract_media(&post);
                let likes = post.favourites_count.unwrap_or(0);
                let shares = post.reblogs_count.unwrap_or(0);
                let url = post
                    .url
                    .clone()
                    .unwrap_or_else(|| self.fallback_post_url(&post));

                results.push(SearchResult {
                    platform: Platform::Mastodon,
                    id: post_id.clone(),
                    author,
                    content: strip_html_tags(post.content.as_deref().unwrap_or("")),
                    created_at,
                    media_urls,
                    media_types,
                    media_count,
                    likes,
                    shares,
                    url,
                });

                processed_any = true;
                max_id = Some(post_id);
            }

            if found_old_post {
                break;
            }

            if !processed_any {
                if let Some(next_id) = fallback_next_max_id {
                    if max_id.as_ref() == Some(&next_id) {
                        break;
                    }
                    max_id = Some(next_id);
                } else {
                    break;
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(results)
    }
}

#[async_trait]
impl SocialPlatform for MastodonService {
    fn platform(&self) -> Platform {
        Platform::Mastodon
    }

    fn is_authenticated(&self) -> bool {
        self.access_token
            .as_ref()
            .map(|token| !token.is_empty())
            .unwrap_or(false)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn instance_url(&self) -> &str {
        &self.instance_url
    }

    async fn search_user(&self, username: &str, days_back: u32) -> Result<Vec<SearchResult>> {
        let cutoff_date = Utc::now() - Duration::days(days_back as i64);
        self.search_user_posts(username, cutoff_date).await
    }

    async fn search_hashtag(&self, hashtag: &str, days_back: u32) -> Result<Vec<SearchResult>> {
        let cutoff_date = Utc::now() - Duration::days(days_back as i64);
        self.search_hashtag_posts(hashtag, cutoff_date).await
    }
}

fn strip_html_tags(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(html, "").trim().to_string()
}
