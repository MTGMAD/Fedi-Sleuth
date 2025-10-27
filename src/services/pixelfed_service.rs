use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;

use crate::models::{AppSettings, PixelfedPost, SearchResult, SearchType};

pub struct PixelfedService {
    client: Client,
    instance_url: String,
    access_token: Option<String>,
}

impl PixelfedService {
    pub fn new(instance_url: String, settings: &AppSettings) -> Self {
        // Ensure the URL has a proper scheme
        let normalized_url =
            if instance_url.starts_with("http://") || instance_url.starts_with("https://") {
                instance_url.trim_end_matches('/').to_string()
            } else {
                format!("https://{}", instance_url.trim_end_matches('/'))
            };

        // Create client with user agent and longer timeout for large hashtag searches
        let client = Client::builder()
            .user_agent("Pixelfed-Rust-Client/0.1.0")
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        let access_token = settings.api.access_token.clone();

        Self {
            client,
            instance_url: normalized_url,
            access_token,
        }
    }

    pub async fn search(
        &self,
        query: String,
        search_type: SearchType,
        days_back: u32,
    ) -> Result<Vec<SearchResult>> {
        // Calculate the cutoff date
        let cutoff_date = Utc::now() - Duration::days(days_back as i64);

        let results = match search_type {
            SearchType::User => self.search_user_posts(&query, cutoff_date).await?,
            SearchType::Hashtag => self.search_hashtag_posts(&query, cutoff_date).await?,
        };

        Ok(results)
    }

    async fn search_user_posts(
        &self,
        username: &str,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>> {
        // Check if we have authentication
        let access_token = self.access_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!(
                "Authentication required. Please enable OAuth in Settings and sign in."
            ))?;

        // Clean username (remove @ if present at the start)
        let clean_username = username.trim_start_matches('@');
        
        // Ensure proper format for federated search
        let search_query = if clean_username.contains('@') {
            // Already has instance, use as-is
            clean_username.to_string()
        } else {
            // Local user, use as-is
            clean_username.to_string()
        };

        // First, search for the user to get their ID
        // Use resolve=true to force federation lookup
        let search_url = format!(
            "{}/api/v2/search?q={}&type=accounts&resolve=true&limit=1",
            self.instance_url, 
            urlencoding::encode(&search_query)
        );

        log::info!("Searching for user '{}' at: {} (this may take a moment for federated lookups...)", search_query, search_url);
        
        let search_response = self.client
            .get(&search_url)
            .header("Authorization", format!("Bearer {}", access_token))
            .timeout(std::time::Duration::from_secs(45))
            .send()
            .await?;

        if !search_response.status().is_success() {
            let status = search_response.status();
            let body = search_response.text().await.unwrap_or_default();
            
            // Check if this is a cross-platform federation issue
            let helpful_msg = if search_query.contains("@mastodon.") || search_query.contains("@fosstodon.") {
                "\n\nNote: Pixelfed instances may have limited federation with Mastodon instances. \
                 Try: 1) Searching on a Mastodon instance instead, or \
                 2) Searching for Pixelfed users only (e.g., from pixelfed.social, pix.art, etc.)"
            } else {
                ""
            };
            
            return Err(anyhow::anyhow!(
                "User search failed: {}. Response: {}. User '{}' may not exist or instance may not be federated.{}",
                status,
                body,
                search_query,
                helpful_msg
            ));
        }

        let search_data: serde_json::Value = search_response.json().await?;
        let accounts = search_data["accounts"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid search response"))?;

        if accounts.is_empty() {
            return Err(anyhow::anyhow!(
                "User '{}' not found. If this is a remote user, their instance may not be federated with {}. Try searching for them directly on their home instance.",
                search_query,
                self.instance_url
            ));
        }

        let user_id = accounts[0]["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid user data"))?;

        // Now fetch the user's posts
        let mut results = Vec::new();
        let mut max_id: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/api/v1/accounts/{}/statuses?limit=40",
                self.instance_url, user_id
            );

            if let Some(ref id) = max_id {
                url.push_str(&format!("&max_id={}", id));
            }

            log::info!("Fetching posts from: {}", url);
            
            let response = self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "Failed to fetch posts: {}",
                    response.status()
                ));
            }

            let posts: Vec<PixelfedPost> = response.json().await?;

            if posts.is_empty() {
                break;
            }

            let mut found_old_post = false;

            for post in posts {
                let created_at = DateTime::parse_from_rfc3339(&post.created_at)?
                    .with_timezone(&Utc);

                if created_at < cutoff_date {
                    found_old_post = true;
                    break;
                }

                let search_result = SearchResult {
                    id: post.id.clone(),
                    author: post.account.username.clone(),
                    content: strip_html_tags(&post.content),
                    created_at,
                    media_urls: post.media_attachments.iter().map(|m| m.url.clone()).collect(),
                    media_types: post.media_attachments.iter().map(|m| m.r#type.clone()).collect(),
                    media_count: post.media_attachments.len() as u32,
                    likes: post.favourites_count,
                    shares: post.reblogs_count,
                    url: post.url,
                };

                results.push(search_result);
                max_id = Some(post.id);
            }

            if found_old_post {
                break;
            }

            // Rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(results)
    }

    async fn search_hashtag_posts(
        &self,
        hashtag: &str,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<SearchResult>> {
        // Check if we have authentication
        let access_token = self.access_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!(
                "Authentication required. Please enable OAuth in Settings and sign in."
            ))?;

        // Clean hashtag (remove # if present)
        let clean_hashtag = hashtag.trim_start_matches('#');

        let mut results = Vec::new();
        let mut max_id: Option<String> = None;

        loop {
            let mut url = format!(
                "{}/api/v1/timelines/tag/{}?limit=40",
                self.instance_url, clean_hashtag
            );

            if let Some(ref id) = max_id {
                url.push_str(&format!("&max_id={}", id));
            }

            log::info!("Fetching hashtag posts from: {}", url);
            
            let response = match self.client
                .get(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .send()
                .await {
                    Ok(resp) => resp,
                    Err(e) => {
                        if e.is_timeout() {
                            return Err(anyhow::anyhow!(
                                "Request timed out while fetching hashtag '{}'. This hashtag may have too many posts. Try searching for a less popular hashtag or a specific user instead.",
                                clean_hashtag
                            ));
                        }
                        return Err(anyhow::anyhow!("Network error: {}", e));
                    }
                };

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(anyhow::anyhow!(
                    "Failed to fetch hashtag posts: {}. Response: {}",
                    status,
                    body
                ));
            }

            let posts: Vec<PixelfedPost> = response.json().await?;

            if posts.is_empty() {
                break;
            }

            let mut found_old_post = false;

            for post in posts {
                let created_at = DateTime::parse_from_rfc3339(&post.created_at)?
                    .with_timezone(&Utc);

                if created_at < cutoff_date {
                    found_old_post = true;
                    break;
                }

                let search_result = SearchResult {
                    id: post.id.clone(),
                    author: post.account.username.clone(),
                    content: strip_html_tags(&post.content),
                    created_at,
                    media_urls: post.media_attachments.iter().map(|m| m.url.clone()).collect(),
                    media_types: post.media_attachments.iter().map(|m| m.r#type.clone()).collect(),
                    media_count: post.media_attachments.len() as u32,
                    likes: post.favourites_count,
                    shares: post.reblogs_count,
                    url: post.url,
                };

                results.push(search_result);
                max_id = Some(post.id);
            }

            if found_old_post {
                break;
            }

            // Rate limiting to avoid overwhelming the API
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(results)
    }
}

fn strip_html_tags(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    re.replace_all(html, "").trim().to_string()
}
