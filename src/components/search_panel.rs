use crate::models::{
    AppSettings, AppState, Platform, PlatformSearchResults, SearchContext, SearchType,
};
use crate::services::{
    platform_display_name, BlueskyService, MastodonService, PixelfedService, SocialPlatform,
};
use dioxus::prelude::*;

fn parse_checkbox(value: &str) -> bool {
    value
        .parse::<bool>()
        .unwrap_or_else(|_| matches!(value, "on" | "1"))
}

#[derive(Clone, Copy, PartialEq)]
struct PlatformSelection {
    pixelfed: bool,
    mastodon: bool,
    bluesky: bool,
}

impl PlatformSelection {
    fn from_settings(settings: &AppSettings) -> Self {
        Self {
            pixelfed: settings.api.pixelfed.enabled,
            mastodon: settings.api.mastodon.enabled,
            bluesky: settings.api.bluesky.enabled,
        }
    }

    fn any(&self) -> bool {
        self.pixelfed || self.mastodon || self.bluesky
    }
}

#[derive(Props, PartialEq)]
pub struct SearchPanelProps {
    pub app_state: UseState<AppState>,
    pub search_results: UseState<Vec<PlatformSearchResults>>,
    pub search_context: UseState<Option<SearchContext>>,
    pub is_searching: UseState<bool>,
    pub status_message: UseState<String>,
}

pub fn SearchPanel(cx: Scope<SearchPanelProps>) -> Element {
    let search_query = use_state(cx, String::new);
    let days_back_input = use_state(cx, || "180".to_string());
    let search_type = use_state(cx, || SearchType::User);
    let selection_overridden = use_state(cx, || false);
    let platform_selection = use_state(cx, || {
        PlatformSelection::from_settings(&cx.props.app_state.get().settings)
    });

    let handle_search = |_| {
        to_owned![
            search_query,
            days_back_input,
            search_type,
            cx.props.search_results,
            cx.props.search_context,
            cx.props.is_searching,
            cx.props.status_message,
            cx.props.app_state,
            platform_selection,
            selection_overridden
        ];

        cx.spawn(async move {
            if search_query.get().trim().is_empty() {
                status_message.set("Please enter a search query".to_string());
                return;
            }

            let input_value = days_back_input.get().clone();
            let parsed_days = input_value.parse::<u32>().unwrap_or(180).clamp(1, 3650);
            let normalized_days = parsed_days.to_string();
            if normalized_days != input_value {
                days_back_input.set(normalized_days);
            }
            let settings_snapshot = app_state.get().settings.clone();
            let default_selection = PlatformSelection::from_settings(&settings_snapshot);
            let selection = if *selection_overridden.get() {
                *platform_selection.get()
            } else {
                default_selection
            };

            if !selection.any() {
                status_message.set("Select at least one platform to search.".to_string());
                is_searching.set(false);
                search_context.set(None);
                return;
            }

            is_searching.set(true);
            status_message.set("Searching...".to_string());
            search_results.set(Vec::new());

            let query_value = search_query.get().clone();
            let search_type_value = search_type.get().clone();
            let context_snapshot =
                SearchContext::new(query_value.clone(), search_type_value.clone(), parsed_days);

            let mut summary_parts: Vec<String> = Vec::new();
            let mut grouped_results: Vec<PlatformSearchResults> = Vec::new();
            let mut total_count: usize = 0;
            let mut any_enabled = false;

            // Pixelfed
            if selection.pixelfed {
                let service = PixelfedService::new(&settings_snapshot);
                let platform = Platform::Pixelfed;
                let label = platform_display_name(platform, service.instance_url());

                if !service.is_enabled() {
                    summary_parts.push(format!("{} disabled", label));
                    grouped_results.push(PlatformSearchResults::error(
                        platform,
                        label,
                        "Disabled in settings".to_string(),
                    ));
                } else {
                    any_enabled = true;
                    match service
                        .search(query_value.clone(), search_type_value.clone(), parsed_days)
                        .await
                    {
                        Ok(mut results) => {
                            results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                            let count = results.len();
                            total_count += count;
                            summary_parts.push(format!("{}: {} posts", label, count));
                            grouped_results
                                .push(PlatformSearchResults::success(platform, label, results));
                        }
                        Err(err) => {
                            let error_msg = err.to_string();
                            summary_parts.push(format!("{} ⚠️ {}", label, error_msg));
                            grouped_results
                                .push(PlatformSearchResults::error(platform, label, error_msg));
                        }
                    }
                }
            } else {
                let label = platform_display_name(
                    Platform::Pixelfed,
                    settings_snapshot.api.pixelfed.instance_url.as_str(),
                );
                summary_parts.push(format!("{} skipped", label));
                grouped_results.push(PlatformSearchResults::error(
                    Platform::Pixelfed,
                    label,
                    "Skipped (not selected)".to_string(),
                ));
            }

            // Mastodon
            if selection.mastodon {
                let service = MastodonService::new(&settings_snapshot);
                let platform = Platform::Mastodon;
                let label = platform_display_name(platform, service.instance_url());

                if !service.is_enabled() {
                    summary_parts.push(format!("{} disabled", label));
                    grouped_results.push(PlatformSearchResults::error(
                        platform,
                        label,
                        "Disabled in settings".to_string(),
                    ));
                } else {
                    any_enabled = true;
                    match service
                        .search(query_value.clone(), search_type_value.clone(), parsed_days)
                        .await
                    {
                        Ok(mut results) => {
                            results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                            let count = results.len();
                            total_count += count;
                            summary_parts.push(format!("{}: {} posts", label, count));
                            grouped_results
                                .push(PlatformSearchResults::success(platform, label, results));
                        }
                        Err(err) => {
                            let error_msg = err.to_string();
                            summary_parts.push(format!("{} ⚠️ {}", label, error_msg));
                            grouped_results
                                .push(PlatformSearchResults::error(platform, label, error_msg));
                        }
                    }
                }
            } else {
                let label = platform_display_name(
                    Platform::Mastodon,
                    settings_snapshot.api.mastodon.instance_url.as_str(),
                );
                summary_parts.push(format!("{} skipped", label));
                grouped_results.push(PlatformSearchResults::error(
                    Platform::Mastodon,
                    label,
                    "Skipped (not selected)".to_string(),
                ));
            }

            // Bluesky
            if selection.bluesky {
                let service = BlueskyService::new(&settings_snapshot);
                let platform = Platform::Bluesky;
                let label = platform_display_name(platform, service.instance_url());

                if !service.is_enabled() {
                    summary_parts.push(format!("{} disabled", label));
                    grouped_results.push(PlatformSearchResults::error(
                        platform,
                        label,
                        "Disabled in settings".to_string(),
                    ));
                } else {
                    any_enabled = true;
                    match service
                        .search(query_value.clone(), search_type_value.clone(), parsed_days)
                        .await
                    {
                        Ok(mut results) => {
                            results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
                            let count = results.len();
                            total_count += count;
                            summary_parts.push(format!("{}: {} posts", label, count));
                            grouped_results
                                .push(PlatformSearchResults::success(platform, label, results));
                        }
                        Err(err) => {
                            let error_msg = err.to_string();
                            summary_parts.push(format!("{} ⚠️ {}", label, error_msg));
                            grouped_results
                                .push(PlatformSearchResults::error(platform, label, error_msg));
                        }
                    }
                }
            } else {
                let label = platform_display_name(Platform::Bluesky, "https://bsky.social");
                summary_parts.push(format!("{} skipped", label));
                grouped_results.push(PlatformSearchResults::error(
                    Platform::Bluesky,
                    label,
                    "Skipped (not selected)".to_string(),
                ));
            }

            search_results.set(grouped_results);

            let summary_suffix = if summary_parts.is_empty() {
                String::new()
            } else {
                format!(" [{}]", summary_parts.join(" | "))
            };

            if any_enabled {
                if total_count > 0 {
                    status_message.set(format!("Fetched {} posts{}", total_count, summary_suffix));
                } else {
                    status_message.set(format!("No posts found{}", summary_suffix));
                }
                search_context.set(Some(context_snapshot));
            } else {
                status_message.set("Selected platforms are disabled in Settings.".to_string());
                search_context.set(None);
            }

            is_searching.set(false);
        });
    };

    let current_selection = if *selection_overridden.get() {
        *platform_selection.get()
    } else {
        PlatformSelection::from_settings(&cx.props.app_state.get().settings)
    };

    cx.render(rsx! {
        div {
            class: "search-form",
            h2 { "Search Configuration" }

            div {
                class: "form-group",
                label { "Search Type:" }
                div {
                    class: "radio-group",
                    label {
                        class: "radio-label",
                        input {
                            r#type: "radio",
                            name: "search_type",
                            checked: matches!(*search_type.get(), SearchType::User),
                            onchange: move |_| search_type.set(SearchType::User),
                        }
                        "👤 User"
                    }
                    label {
                        class: "radio-label",
                        input {
                            r#type: "radio",
                            name: "search_type",
                            checked: matches!(*search_type.get(), SearchType::Hashtag),
                            onchange: move |_| search_type.set(SearchType::Hashtag),
                        }
                        "#️⃣ Hashtag"
                    }
                }
            }

            div {
                class: "form-group",
                label {
                    match *search_type.get() {
                        SearchType::User => "Username:",
                        SearchType::Hashtag => "Hashtag:",
                    }
                }
                input {
                    r#type: "text",
                    value: "{search_query}",
                    placeholder: match *search_type.get() {
                        SearchType::User => "@username",
                        SearchType::Hashtag => "#hashtag",
                    },
                    oninput: move |evt| search_query.set(evt.value.clone()),
                }
            }

            div {
                class: "form-group",
                label { "Days to search back:" }
                input {
                    r#type: "number",
                    value: "{days_back_input}",
                    min: "1",
                    max: "3650",
                    oninput: move |evt| {
                        days_back_input.set(evt.value.clone());
                    },
                }
                small { "Default: 180 days (about 6 months)" }
            }

            div {
                class: "form-group",
                label { "Platforms to search:" }
                div {
                    class: "checkbox-group",
                    label {
                        class: "checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: current_selection.pixelfed,
                            onchange: move |evt| {
                                selection_overridden.set(true);
                                let mut selection = *platform_selection.get();
                                selection.pixelfed = parse_checkbox(&evt.value);
                                platform_selection.set(selection);
                            },
                        }
                        "Pixelfed"
                    }
                    label {
                        class: "checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: current_selection.mastodon,
                            onchange: move |evt| {
                                selection_overridden.set(true);
                                let mut selection = *platform_selection.get();
                                selection.mastodon = parse_checkbox(&evt.value);
                                platform_selection.set(selection);
                            },
                        }
                        "Mastodon"
                    }
                    label {
                        class: "checkbox-label",
                        input {
                            r#type: "checkbox",
                            checked: current_selection.bluesky,
                            onchange: move |evt| {
                                selection_overridden.set(true);
                                let mut selection = *platform_selection.get();
                                selection.bluesky = parse_checkbox(&evt.value);
                                platform_selection.set(selection);
                            },
                        }
                        "Bluesky"
                    }
                }
                small { "Toggle platforms per search. Configure credentials in Settings." }
            }

            button {
                class: "search-btn primary",
                disabled: *cx.props.is_searching.get(),
                onclick: handle_search,
                if *cx.props.is_searching.get() {
                    "🔄 Searching..."
                } else {
                    "🔍 Start Search"
                }
            }
        }
    })
}
