use crate::models::{AppState, SearchResult, SearchType};
use crate::services::PixelfedService;
use dioxus::prelude::*;

#[derive(Props, PartialEq)]
pub struct SearchPanelProps {
    pub app_state: UseState<AppState>,
    pub search_results: UseState<Vec<SearchResult>>,
    pub is_searching: UseState<bool>,
    pub status_message: UseState<String>,
}

pub fn SearchPanel(cx: Scope<SearchPanelProps>) -> Element {
    let search_query = use_state(cx, String::new);
    let days_back = use_state(cx, || 180u32);
    let search_type = use_state(cx, || SearchType::User);
    let instance_url = use_state(cx, || "pixelfed.social".to_string());

    let handle_search = |_| {
        to_owned![
            search_query,
            days_back,
            search_type,
            instance_url,
            cx.props.search_results,
            cx.props.is_searching,
            cx.props.status_message,
            cx.props.app_state
        ];

        cx.spawn(async move {
            if search_query.get().trim().is_empty() {
                status_message.set("Please enter a search query".to_string());
                return;
            }

            is_searching.set(true);
            status_message.set("Searching...".to_string());
            search_results.set(Vec::new());

            let service =
                PixelfedService::new(instance_url.get().clone(), &app_state.get().settings);

            match service
                .search(
                    search_query.get().clone(),
                    search_type.get().clone(),
                    *days_back.get(),
                )
                .await
            {
                Ok(results) => {
                    let result_count = results.len();
                    search_results.set(results);
                    status_message.set(format!("Found {} results", result_count));
                }
                Err(e) => {
                    status_message.set(format!("Search failed: {}", e));
                }
            }

            is_searching.set(false);
        });
    };

    cx.render(rsx! {
        div {
            class: "search-form",
            h2 { "Search Configuration" }

            div {
                class: "form-group",
                label { "Instance URL:" }
                input {
                    r#type: "text",
                    value: "{instance_url}",
                    placeholder: "pixelfed.social",
                    oninput: move |evt| instance_url.set(evt.value.clone()),
                }
            }

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
                    value: "{days_back}",
                    min: "1",
                    max: "3650",
                    oninput: move |evt| {
                        if let Ok(val) = evt.value.parse::<u32>() {
                            days_back.set(val);
                        }
                    },
                }
                small { "Default: 180 days (about 6 months)" }
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
