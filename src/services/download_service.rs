use anyhow::Result;
use chrono::Utc;
use futures_util::StreamExt;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

use crate::models::{AppSettings, SearchResult};

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
        results: Vec<SearchResult>,
        mut progress_callback: F,
    ) -> Result<PathBuf>
    where
        F: FnMut(f64),
    {
        if results.is_empty() {
            return Err(anyhow::anyhow!("No results to download"));
        }

        // Create download directory
        let download_dir = self.create_download_directory(&results[0])?;

        let total_files: usize = results.iter().map(|r| r.media_urls.len()).sum();
        let mut downloaded_files = 0;

        progress_callback(0.0);

        // Use semaphore to limit concurrent downloads
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
            self.settings.download.max_concurrent as usize,
        ));

        // Create tasks for all downloads
        let mut tasks = Vec::new();

        for result in results {
            for (media_index, media_url) in result.media_urls.iter().enumerate() {
                let permit = semaphore.clone().acquire_owned().await?;
                let client = self.client.clone();
                let download_dir = download_dir.clone();
                let media_url = media_url.clone();
                let result_id = result.id.clone();

                let task = tokio::spawn(async move {
                    let _permit = permit; // Hold permit until task completes

                    let filename = Self::generate_filename(&result_id, media_index, &media_url);
                    let file_path = download_dir.join(filename);

                    Self::download_file(&client, &media_url, &file_path).await
                });

                tasks.push(task);
            }
        }

        // Wait for all downloads to complete
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

        Ok(download_dir)
    }

    fn create_download_directory(&self, first_result: &SearchResult) -> Result<PathBuf> {
        let base_path = Path::new(&self.settings.download.base_path);
        let pixelfed_dir = base_path.join("pixelfed");

        let folder_name = if self.settings.download.organize_by_date {
            format!("{}_{}", first_result.author, Utc::now().format("%Y-%m-%d"))
        } else {
            first_result.author.clone()
        };

        let download_dir = pixelfed_dir.join(folder_name);
        fs::create_dir_all(&download_dir)?;

        Ok(download_dir)
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
