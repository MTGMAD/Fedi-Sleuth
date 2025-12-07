use crate::models::{AppState, PlatformSearchResults, SearchContext, SearchType};
use crate::services::DownloadService;
use dioxus::prelude::*;

#[derive(Props, PartialEq)]
pub struct OutputPanelProps {
    pub search_results: UseState<Vec<PlatformSearchResults>>,
    pub app_state: UseState<AppState>,
    pub search_context: UseState<Option<SearchContext>>,
    pub status_message: UseState<String>,
}

pub fn OutputPanel(cx: Scope<OutputPanelProps>) -> Element {
    let is_downloading = use_state(cx, || false);
    let download_progress = use_state(cx, || 0.0f64);

    let handle_download = |_| {
        to_owned![
            cx.props.search_results,
            cx.props.search_context,
            cx.props.app_state,
            cx.props.status_message,
            is_downloading,
            download_progress
        ];

        cx.spawn(async move {
            let current_groups = search_results.get().clone();
            let has_media = current_groups.iter().any(|group| !group.results.is_empty());

            if !has_media {
                status_message.set("No results to download".to_string());
                return;
            }

            is_downloading.set(true);
            status_message.set("Starting download...".to_string());

            let download_service = DownloadService::new(app_state.get().settings.clone());
            let context_snapshot = search_context.get().clone();

            match download_service
                .download_all(context_snapshot, current_groups, |progress| {
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
                let total_posts: usize = cx
                    .props
                    .search_results
                    .get()
                    .iter()
                    .map(|group| group.results.len())
                    .sum();
                let successful_platforms = cx
                    .props
                    .search_results
                    .get()
                    .iter()
                    .filter(|group| group.error.is_none())
                    .count();
                let error_platforms = cx
                    .props
                    .search_results
                    .get()
                    .iter()
                    .filter(|group| group.error.is_some())
                    .count();

                rsx! {
                    div {
                        class: "results-summary",
                        p { "{total_posts} posts across {successful_platforms} platform(s)" }

                        if let Some(context) = cx.props.search_context.get().as_ref() {
                            let label = match context.search_type {
                                SearchType::User => format!("User: {}", context.query),
                                SearchType::Hashtag => format!("Hashtag: {}", context.query),
                            };
                            rsx! {
                                small {
                                    class: "summary-context",
                                    "{label} · Last {context.days_back} day(s)"
                                }
                            }
                        }

                        if error_platforms > 0 {
                            rsx! {
                                small {
                                    class: "summary-errors",
                                    "{error_platforms} platform(s) reported an error"
                                }
                            }
                        }

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
                        class: "results-groups",
                        cx.props.search_results.get().iter().enumerate().map(|(group_index, group)| rsx! {
                            div {
                                key: "{group_index}",
                                class: "platform-section",
                                div {
                                    class: "platform-header",
                                    h3 { "{group.label}" }
                                    span { class: "platform-count", "{group.results.len()} posts" }
                                }

                                if let Some(error) = &group.error {
                                    rsx! {
                                        div {
                                            class: "platform-error",
                                            "⚠️ {error}"
                                        }
                                    }
                                } else if group.results.is_empty() {
                                    rsx! {
                                        div {
                                            class: "platform-empty",
                                            "No posts returned from this platform."
                                        }
                                    }
                                } else {
                                    rsx! {
                                        div {
                                            class: "platform-results",
                                            group.results.iter().enumerate().map(|(index, result)| rsx! {
                                                div {
                                                    key: "{index}",
                                                    class: "result-item",
                                                    div {
                                                        class: "result-header",
                                                        span {
                                                            class: "result-platform",
                                                            "{group.platform.emoji()} {group.platform.name()}"
                                                        }
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
                }
            }
        }
    })
}
