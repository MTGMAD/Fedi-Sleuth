use crate::components::{OutputPanel, SearchPanel, SettingsPanel, StatusBar};
use crate::models::{AppState, PlatformSearchResults, SearchContext};
use crate::services::SettingsService;
use dioxus::prelude::*;

#[derive(Props, PartialEq)]
pub struct AppProps {}

pub fn App(cx: Scope<AppProps>) -> Element {
    // Initialize app state
    let app_state = use_state(cx, || AppState::default());
    let current_view = use_state(cx, || "search");
    let search_results = use_state(cx, || Vec::<PlatformSearchResults>::new());
    let search_context = use_state(cx, || None::<SearchContext>);
    let is_searching = use_state(cx, || false);
    let status_message = use_state(cx, || String::new());

    // Load settings on startup
    use_effect(cx, (), |_| {
        to_owned![app_state];
        async move {
            if let Ok(settings) = SettingsService::load_settings().await {
                app_state.set(AppState { settings });
            }
        }
    });

    cx.render(rsx! {
        style { include_str!("../assets/styles.css") }
        div {
            class: "app-container",
            "data-theme": "{app_state.settings.appearance.theme}",
            style: "--accent-color: {app_state.settings.appearance.accent_color}",

            // Header with navigation
            header {
                class: "app-header",
                h1 { class: "app-title", "Fedi Sleuth" }
                div {
                    class: "nav-buttons",
                    button {
                        class: if **current_view == "search" { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| current_view.set("search"),
                        "🔍 Search"
                    }
                    button {
                        class: if **current_view == "settings" { "nav-btn active" } else { "nav-btn" },
                        onclick: move |_| current_view.set("settings"),
                        "⚙️ Settings"
                    }
                }
            }

            // Main content area
            main {
                class: "main-content",
                match **current_view {
                    "search" => rsx! {
                        div {
                            class: "search-layout",
                            div {
                                class: "search-panel",
                                SearchPanel {
                                    app_state: app_state.clone(),
                                    search_results: search_results.clone(),
                                    search_context: search_context.clone(),
                                    is_searching: is_searching.clone(),
                                    status_message: status_message.clone(),
                                }
                            }
                            div {
                                class: "output-panel",
                                OutputPanel {
                                    search_results: search_results.clone(),
                                    app_state: app_state.clone(),
                                    search_context: search_context.clone(),
                                    status_message: status_message.clone(),
                                }
                            }
                        }
                    },
                    "settings" => rsx! {
                        SettingsPanel {
                            app_state: app_state.clone(),
                        }
                    },
                    _ => rsx! { div { "Unknown view" } }
                }
            }

            // Status bar
            StatusBar {
                message: (**status_message).clone(),
                is_searching: **is_searching,
            }
        }
    })
}
