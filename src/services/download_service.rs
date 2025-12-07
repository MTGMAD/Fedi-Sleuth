use anyhow::Result;
use chrono::Utc;
use futures_util::StreamExt;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::models::{AppSettings, PlatformSearchResults, SearchContext, SearchResult, SearchType};

pub struct DownloadService {
    client: reqwest::Client,
    settings: AppSettings,
}

impl DownloadService {
    pub fn new(settings: AppSettings) -> Self {
        Self {
            client: reqwest::Client::new(),
            settings,
        }
    }

    pub async fn download_all<F>(
        &self,
        context: Option<SearchContext>,
        groups: Vec<PlatformSearchResults>,
        mut progress_callback: F,
    ) -> Result<PathBuf>
    where
        F: FnMut(f64),
    {
        let mut results: Vec<SearchResult> = Vec::new();

        for group in groups.into_iter() {
            if group.error.is_some() {
                continue;
            }
            results.extend(group.results.into_iter());
        }

        if results.is_empty() {
            return Err(anyhow::anyhow!("No results to download"));
        }

        let total_files: usize = results.iter().map(|result| result.media_urls.len()).sum();
        if total_files == 0 {
            return Err(anyhow::anyhow!("No media attachments to download"));
        }

        let download_root = self.create_download_root(context.as_ref())?;
        let mut ensured_dirs: HashSet<PathBuf> = HashSet::new();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
            self.settings.download.max_concurrent as usize,
        ));

        progress_callback(0.0);

        let mut tasks = Vec::new();

        for result in results {
            if result.media_urls.is_empty() {
                continue;
            }

            let platform_dir = download_root.join(result.platform.folder_name());
            if ensured_dirs.insert(platform_dir.clone()) {
                fs::create_dir_all(&platform_dir)?;
            }

            for (media_index, media_url) in result.media_urls.iter().enumerate() {
                let permit = semaphore.clone().acquire_owned().await?;
                let client = self.client.clone();
                let media_url = media_url.clone();
                let file_dir = platform_dir.clone();
                let result_id = result.id.clone();

                let task = tokio::spawn(async move {
                    let _permit = permit;
                    if let Err(err) = tokio::fs::create_dir_all(&file_dir).await {
                        return Err(anyhow::anyhow!(
                            "Failed to prepare download directory: {}",
                            err
                        ));
                    }

                    let filename = Self::generate_filename(&result_id, media_index, &media_url);
                    let file_path = file_dir.join(filename);

                    Self::download_file(&client, &media_url, &file_path).await
                });

                tasks.push(task);
            }
        }

        let mut downloaded_files = 0usize;

        for task in tasks {
            match task.await? {
                Ok(_) => {
                    downloaded_files += 1;
                    let progress = downloaded_files as f64 / total_files as f64;
                    progress_callback(progress);
                }
                Err(e) => {
                    log::warn!("Failed to download file: {}", e);
                }
            }
        }

        Ok(download_root)
    }

    fn create_download_root(&self, context: Option<&SearchContext>) -> Result<PathBuf> {
        let base_path = Path::new(&self.settings.download.base_path);
        let now = Utc::now();

        let mut root = if self.settings.download.organize_by_date {
            base_path.join(now.format("%Y-%m-%d").to_string())
        } else {
            base_path.to_path_buf()
        };

        let (query_folder, days_segment) = context
            .map(|ctx| {
                let prefix = match ctx.search_type {
                    SearchType::User => "user",
                    SearchType::Hashtag => "hashtag",
                };
                (
                    format!("{}-{}", prefix, ctx.get_folder_name()),
                    format!("{}d", ctx.days_back),
                )
            })
            .unwrap_or_else(|| ("search".to_string(), "any".to_string()));

        root = root.join(format!(
            "{}-{}-{}",
            query_folder,
            days_segment,
            now.format("%H%M%S")
        ));

        fs::create_dir_all(&root)?;

        Ok(root)
    }

    async fn download_file(client: &reqwest::Client, url: &str, file_path: &Path) -> Result<()> {
        let response = client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download: HTTP {}",
                response.status()
            ));
        }

        let mut file = tokio::fs::File::create(file_path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;
        Ok(())
    }

    fn generate_filename(post_id: &str, media_index: usize, url: &str) -> String {
        // Extract file extension from URL
        let extension = Path::new(url)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("jpg");

        format!("{}_{:03}.{}", post_id, media_index + 1, extension)
    }
}
