use crate::models::{AppState, SearchResult};
use crate::services::DownloadService;
use dioxus::prelude::*;

#[derive(Props, PartialEq)]
pub struct OutputPanelProps {
    pub search_results: UseState<Vec<SearchResult>>,
    pub app_state: UseState<AppState>,
    pub status_message: UseState<String>,
}

pub fn OutputPanel(cx: Scope<OutputPanelProps>) -> Element {
    let is_downloading = use_state(cx, || false);
    let download_progress = use_state(cx, || 0.0f64);

    let handle_download = |_| {
        to_owned![
            cx.props.search_results,
            cx.props.app_state,
            cx.props.status_message,
            is_downloading,
            download_progress
        ];

        cx.spawn(async move {
            if search_results.get().is_empty() {
                status_message.set("No results to download".to_string());
                return;
            }

            is_downloading.set(true);
            status_message.set("Starting download...".to_string());

            let download_service = DownloadService::new(app_state.get().settings.clone());

            match download_service
                .download_all(search_results.get().clone(), |progress| {
                    download_progress.set(progress);
                    status_message.set(format!("Downloading... {:.1}%", progress * 100.0));
                })
                .await
            {
                Ok(download_path) => {
                    status_message.set(format!(
                        "Download completed! Files saved to: {}",
                        download_path.display()
                    ));
                }
                Err(e) => {
                    status_message.set(format!("Download failed: {}", e));
                }
            }

            is_downloading.set(false);
            download_progress.set(0.0);
        });
    };

    cx.render(rsx! {
        div {
            class: "output-container",
            h2 { "Search Results" }

            if cx.props.search_results.get().is_empty() {
                rsx! {
                    div {
                        class: "empty-state",
                        "🔍 No results yet. Use the search panel to find users or hashtags."
                    }
                }
            } else {
                rsx! {
                    div {
                        class: "results-summary",
                        p { "Found {cx.props.search_results.get().len()} posts" }
                        
                        button {
                            class: "download-btn primary",
                            disabled: *is_downloading.get(),
                            onclick: handle_download,
                            if *is_downloading.get() {
                                "⬇️ Downloading... {download_progress.get() * 100.0:.1}%"
                            } else {
                                "⬇️ Download All"
                            }
                        }

                        if *is_downloading.get() {
                            rsx! {
                                div {
                                    class: "progress-bar",
                                    div {
                                        class: "progress-fill",
                                        style: "width: {download_progress.get() * 100.0}%",
                                    }
                                }
                            }
                        }
                    }

                    div {
                        class: "results-list",
                        cx.props.search_results.get().iter().enumerate().map(|(index, result)| rsx! {
                            div {
                                key: "{index}",
                                class: "result-item",
                                div {
                                    class: "result-header",
                                    strong { "{result.author}" }
                                    span { class: "result-date", "{result.created_at}" }
                                }
                                (!result.content.is_empty()).then(|| rsx! {
                                    p { class: "result-content", "{result.content}" }
                                })
                                div {
                                    class: "result-meta",
                                    span { "📷 {result.media_count} media" }
                                    span { "👍 {result.likes}" }
                                    span { "🔄 {result.shares}" }
                                }
                                
                                // Hover popup
                                div {
                                    class: "result-popup",
                                    div { class: "popup-header",
                                        strong { "{result.author}" }
                                        span { "{result.created_at}" }
                                    }
                                    if !result.content.is_empty() {
                                        rsx! {
                                            div { class: "popup-content", "{result.content}" }
                                        }
                                    }
                                    if !result.media_urls.is_empty() {
                                        rsx! {
                                            div { class: "popup-media",
                                                result.media_urls.iter().zip(result.media_types.iter()).enumerate().map(|(idx, (url, media_type))| {
                                                    if media_type == "video" || media_type == "gifv" {
                                                        rsx! {
                                                            video {
                                                                key: "{url}",
                                                                class: "popup-thumbnail",
                                                                src: "{url}",
                                                                controls: "true",
                                                                preload: "metadata",
                                                                muted: "true",
                                                                r#loop: "true",
                                                                playsinline: "true",
                                                            }
                                                        }
                                                    } else {
                                                        rsx! {
                                                            img {
                                                                key: "{url}",
                                                                class: "popup-thumbnail",
                                                                src: "{url}",
                                                                alt: "Media {idx + 1}"
                                                            }
                                                        }
                                                    }
                                                })
                                            }
                                        }
                                    }
                                    div { class: "popup-meta",
                                        div { "📷 Media: {result.media_count}" }
                                        div { "👍 Likes: {result.likes}" }
                                        div { "🔄 Shares: {result.shares}" }
                                    }
                                }
                            }
                        })
                    }
                }
            }
        }
    })
}
