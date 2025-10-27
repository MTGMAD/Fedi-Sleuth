use dioxus::prelude::*;
use oauth2::AuthorizationCode;
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};
use url::Url;

use crate::models::{ApiSettings, AppSettings, AppState, Theme};
use crate::services::{AuthService, SettingsService};

// OAuth helper functions
async fn start_oauth_flow(app_settings: AppSettings) -> Result<ApiSettings, String> {
    let instance_url = normalize_instance_url(&app_settings.instance_url)?;

    let mut api_settings = app_settings.api.clone();
    api_settings.use_oauth = true;

    // Start the callback listener FIRST to get a free port
    log::info!("Starting OAuth callback listener on a free port...");
    
    let listener = TcpListener::bind(("127.0.0.1", 0)).await
        .map_err(|e| format!("Failed to start callback listener: {}", e))?;
    
    let callback_port = listener.local_addr()
        .map_err(|e| format!("Failed to get listener address: {}", e))?
        .port();
    
    log::info!("Callback listener ready on port {}", callback_port);

    // Always register a new OAuth app with the dynamic port
    let redirect_uri = format!("http://localhost:{}/callback", callback_port);
    
    let registration_service = AuthService::new_with_redirect(
        api_settings.clone(), 
        &instance_url,
        &redirect_uri
    ).map_err(|e| format!("Failed to initialize auth service: {}", e))?;

    let (client_id, client_secret) = registration_service
        .register_app(&api_settings.app_name)
        .await
        .map_err(|e| format!("Failed to register OAuth app: {}", e))?;

    api_settings.client_id = client_id;
    api_settings.client_secret = client_secret;

    let oauth_service = AuthService::new_with_redirect(
        api_settings.clone(), 
        &instance_url,
        &redirect_uri
    ).map_err(|e| format!("Failed to initialize OAuth client: {}", e))?;

    let (auth_url, csrf_token) = oauth_service
        .generate_auth_url()
        .map_err(|e| format!("Failed to generate authorization URL: {}", e))?;

    log::info!("Opening browser for OAuth authorization...");

    // NOW open the browser
    open_browser(auth_url.as_str())?;
    log::info!("Opened browser for OAuth authorization: {}", auth_url);

    // Wait for the callback
    let callback_result = wait_for_oauth_callback_with_listener(listener).await?;
    let (code, state) = callback_result;

    if state != csrf_token.secret().as_str() {
        return Err("OAuth state mismatch. Please try again.".to_string());
    }

    let access_token = oauth_service
        .exchange_code(AuthorizationCode::new(code), csrf_token)
        .await
        .map_err(|e| format!("Failed to exchange authorization code: {}", e))?;

    api_settings.access_token = Some(access_token);

    Ok(api_settings)
}

fn open_browser(url: &str) -> Result<(), String> {
    // Use the system's default browser to open the URL
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("rundll32")
            .arg("url.dll,FileProtocolHandler")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open browser: {}", e))?;
    }

    Ok(())
}

fn normalize_instance_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Instance URL is empty. Please configure your Pixelfed instance.".to_string());
    }

    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{}", trimmed)
    };

    Ok(with_scheme.trim_end_matches('/').to_string())
}

async fn wait_for_oauth_callback_with_listener(listener: TcpListener) -> Result<(String, String), String> {
    let timeout_duration = Duration::from_secs(180);

    log::info!("Waiting for OAuth callback on http://localhost:8080/callback ...");

    let (mut stream, addr) = timeout(timeout_duration, listener.accept())
        .await
        .map_err(|_| "OAuth authorization timed out after 3 minutes. Please try again.".to_string())?
        .map_err(|e| format!("Failed to accept OAuth callback: {}", e))?;

    log::info!("Received connection from: {}", addr);

    let mut buffer = vec![0u8; 4096];
    let bytes_read = timeout(timeout_duration, stream.read(&mut buffer))
        .await
        .map_err(|_| "OAuth callback read timed out. Please try again.".to_string())?
        .map_err(|e| format!("Failed to read OAuth callback: {}", e))?;

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    log::debug!("Received OAuth callback request: {}", request.lines().next().unwrap_or(""));
    
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| "Invalid OAuth callback request.".to_string())?;

    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "Invalid OAuth callback request.".to_string())?;

    let url = Url::parse(&format!("http://localhost{}", path))
        .map_err(|e| format!("Failed to parse OAuth callback URL: {}", e))?;

    let mut code = None;
    let mut state = None;
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            _ => {}
        }
    }

    let response_body = "<!DOCTYPE html><html><head><title>Authentication Successful</title><style>body{font-family:Arial,sans-serif;display:flex;justify-content:center;align-items:center;height:100vh;margin:0;background:#f0f0f0;}div{text-align:center;padding:40px;background:white;border-radius:8px;box-shadow:0 2px 10px rgba(0,0,0,0.1);}</style></head><body><div><h1 style='color:#4CAF50;'>✓ Authentication Successful!</h1><p>You can close this window and return to the application.</p></div></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );

    timeout(timeout_duration, stream.write_all(response.as_bytes()))
        .await
        .map_err(|_| "OAuth callback response timed out.".to_string())?
        .map_err(|e| format!("Failed to send OAuth callback response: {}", e))?;

    let _ = stream.shutdown().await;

    let code = code.ok_or_else(|| "Missing authorization code in callback.".to_string())?;
    let state = state.ok_or_else(|| "Missing OAuth state in callback.".to_string())?;

    log::info!("OAuth callback received successfully");

    Ok((code, state))
}

#[derive(Props, PartialEq)]
pub struct SettingsPanelProps {
    pub app_state: UseState<AppState>,
}

pub fn SettingsPanel(cx: Scope<SettingsPanelProps>) -> Element {
    let temp_settings = use_state(cx, || cx.props.app_state.current().settings.clone());
    let current_section = use_state(cx, || "appearance");

    let save_settings = |_| {
        to_owned![temp_settings, cx.props.app_state];
        cx.spawn(async move {
            let settings_to_save = temp_settings.current().as_ref().clone();
            if let Err(e) = SettingsService::save_settings(&settings_to_save).await {
                log::error!("Failed to save settings: {}", e);
                return;
            }

            app_state.set(AppState {
                settings: settings_to_save,
            });
        });
    };

    cx.render(rsx! {
        div {
            class: "settings-layout",
            
            // Left sidebar navigation
            div {
                class: "settings-sidebar",
                h2 { "Settings" }
                
                div {
                    class: "settings-nav",
                    button {
                        class: if **current_section == "appearance" { "settings-nav-btn active" } else { "settings-nav-btn" },
                        onclick: move |_| current_section.set("appearance"),
                        "🎨 Appearance"
                    }
                    button {
                        class: if **current_section == "api" { "settings-nav-btn active" } else { "settings-nav-btn" },
                        onclick: move |_| current_section.set("api"),
                        "🔑 API & Authentication"
                    }
                    button {
                        class: if **current_section == "download" { "settings-nav-btn active" } else { "settings-nav-btn" },
                        onclick: move |_| current_section.set("download"),
                        "📁 Download Settings"
                    }
                }
            }
            
            // Right content area
            div {
                class: "settings-content",
                match **current_section {
                    "appearance" => rsx! {
                        div {
                            class: "settings-section-content",
                            h3 { "🎨 Appearance Settings" }
                            
                            div {
                                class: "form-group",
                                label { "Theme:" }
                                select {
                                    value: "{temp_settings.current().appearance.theme}",
                                    onchange: move |evt| {
                                        let mut settings = temp_settings.current().as_ref().clone();
                                        settings.appearance.theme = match evt.value.as_str() {
                                            "light" => Theme::Light,
                                            "dark" => Theme::Dark,
                                            _ => Theme::System,
                                        };
                                        temp_settings.set(settings);
                                    },
                                    option { value: "system", "🖥️ System" }
                                    option { value: "light", "☀️ Light" }
                                    option { value: "dark", "🌙 Dark" }
                                }
                            }

                            div {
                                class: "form-group",
                                label { "Accent Color:" }
                                div {
                                    class: "color-picker-grid",
                                    ["#0078d4", "#107c10", "#d83b01", "#b146c2", "#881798", "#e3008c", "#00bcf2", "#008272"].iter().map(|color| rsx! {
                                        button {
                                            key: "{color}",
                                            class: if temp_settings.current().appearance.accent_color == *color { "color-btn active" } else { "color-btn" },
                                            style: "background-color: {color}",
                                            onclick: move |_| {
                                                let mut settings = temp_settings.current().as_ref().clone();
                                                settings.appearance.accent_color = color.to_string();
                                                temp_settings.set(settings);
                                            },
                                            ""
                                        }
                                    })
                                }
                                input {
                                    r#type: "color",
                                    value: "{temp_settings.current().appearance.accent_color}",
                                    onchange: move |evt| {
                                        let mut settings = temp_settings.current().as_ref().clone();
                                        settings.appearance.accent_color = evt.value.clone();
                                        temp_settings.set(settings);
                                    },
                                }
                            }
                        }
                    },
                    "api" => rsx! {
                        div {
                            class: "settings-section-content",
                            h3 { "🔑 API & Authentication Settings" }
                            
                            div {
                                class: "form-group",
                                label { "Pixelfed Instance URL:" }
                                input {
                                    r#type: "text",
                                    value: "{temp_settings.current().instance_url}",
                                    placeholder: "pixelfed.social",
                                    oninput: move |evt| {
                                        let mut settings = temp_settings.current().as_ref().clone();
                                        settings.instance_url = evt.value.clone();
                                        temp_settings.set(settings);
                                    },
                                }
                                small { "Enter the URL of your Pixelfed instance (without https://)" }
                            }
                            
                            div {
                                class: "form-group",
                                label { "API Access Method:" }
                                div {
                                    class: "radio-group",
                                    label {
                                        class: "radio-label",
                                        input {
                                            r#type: "radio",
                                            name: "api_method",
                                            checked: !temp_settings.current().api.use_oauth,
                                            onchange: move |_| {
                                                let mut settings = temp_settings.current().as_ref().clone();
                                                settings.api.use_oauth = false;
                                                temp_settings.set(settings);
                                            },
                                        }
                                        "🌐 Public API (Limited Access)"
                                    }
                                    label {
                                        class: "radio-label",
                                        input {
                                            r#type: "radio",
                                            name: "api_method",
                                            checked: temp_settings.current().api.use_oauth,
                                            onchange: move |_| {
                                                let mut settings = temp_settings.current().as_ref().clone();
                                                settings.api.use_oauth = true;
                                                temp_settings.set(settings);
                                            },
                                        }
                                        "🔑 OAuth Authentication (Full Access)"
                                    }
                                }
                                small { 
                                    if temp_settings.current().api.use_oauth {
                                        "OAuth provides full API access and higher rate limits, but requires authentication."
                                    } else {
                                        "Public API allows basic searches without authentication, but with limited access and lower rate limits."
                                    }
                                }
                            }

                            if temp_settings.current().api.use_oauth {
                                rsx! {
                                    div {
                                        class: "oauth-section",
                                        
                                        div {
                                            class: "form-group",
                                            label { "OAuth Application Name:" }
                                            input {
                                                r#type: "text",
                                                value: "{temp_settings.current().api.app_name}",
                                                placeholder: "Pixelfed Rust Client",
                                                oninput: move |evt| {
                                                    let mut settings = temp_settings.current().as_ref().clone();
                                                    settings.api.app_name = evt.value.clone();
                                                    temp_settings.set(settings);
                                                },
                                            }
                                            small { "Name for your OAuth application (shown to users during login)" }
                                        }
                                        
                                        div {
                                            class: "form-group",
                                            label { "Client ID:" }
                                            input {
                                                r#type: "text",
                                                value: "{temp_settings.current().api.client_id}",
                                                placeholder: "Your OAuth Client ID",
                                                oninput: move |evt| {
                                                    let mut settings = temp_settings.current().as_ref().clone();
                                                    settings.api.client_id = evt.value.clone();
                                                    temp_settings.set(settings);
                                                },
                                            }
                                        }
                                        div {
                                            class: "form-group",
                                            label { "Client Secret:" }
                                            input {
                                                r#type: "password",
                                                value: "{temp_settings.current().api.client_secret}",
                                                placeholder: "Your OAuth Client Secret",
                                                oninput: move |evt| {
                                                    let mut settings = temp_settings.current().as_ref().clone();
                                                    settings.api.client_secret = evt.value.clone();
                                                    temp_settings.set(settings);
                                                },
                                            }
                                        }
                                        
                                        div {
                                            class: "oauth-status",
                                            if !temp_settings.current().api.access_token.is_none() {
                                                rsx! {
                                                    p { 
                                                        style: "color: var(--success); font-weight: 500;",
                                                        "✅ Authenticated - OAuth token active"
                                                    }
                                                    button {
                                                        class: "oauth-btn secondary",
                                                        onclick: move |_| {
                                                            let mut settings = temp_settings.current().as_ref().clone();
                                                            settings.api.access_token = None;
                                                            settings.api.client_id = String::new();
                                                            settings.api.client_secret = String::new();
                                                            temp_settings.set(settings);
                                                        },
                                                        "🚪 Sign Out & Clear Credentials"
                                                    }
                                                }
                                            } else {
                                                rsx! {
                                                    p { 
                                                        style: "color: var(--info); font-weight: 500;",
                                                        "🔑 Ready to authenticate with Pixelfed"
                                                    }
                                                    button {
                                                        class: "oauth-btn primary",
                                                        onclick: move |_| {
                                                            to_owned![temp_settings, cx.props.app_state];

                                                            cx.spawn(async move {
                                                                log::info!("Starting OAuth flow...");

                                                                let current_settings = temp_settings.current().as_ref().clone();

                                                                match start_oauth_flow(current_settings.clone()).await {
                                                                    Ok(updated_api) => {
                                                                        let mut merged_settings = current_settings;
                                                                        merged_settings.api = updated_api;

                                                                        temp_settings.set(merged_settings.clone());

                                                                        app_state.set(AppState {
                                                                            settings: merged_settings.clone(),
                                                                        });

                                                                        if let Err(err) = SettingsService::save_settings(&merged_settings).await {
                                                                            log::error!("Failed to persist OAuth settings: {}", err);
                                                                        } else {
                                                                            log::info!("OAuth authentication completed successfully");
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        log::error!("OAuth setup failed: {}", e);
                                                                    }
                                                                }
                                                            });
                                                        },
                                                        "🔑 Sign In with Pixelfed"
                                                    }
                                                    
                                                    if !temp_settings.current().api.client_id.is_empty() {
                                                        rsx! {
                                                            div {
                                                                style: "margin-top: 12px;",
                                                                p { 
                                                                    style: "color: var(--text-secondary); font-size: 13px;",
                                                                    "App registered. Click above to complete authorization in your browser."
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        div {
                                            class: "oauth-help",
                                            h4 { "How to get OAuth credentials:" }
                                            ol {
                                                li { "Go to your Pixelfed instance (e.g., pixelfed.social)" }
                                                li { "Navigate to Settings → Applications → Developer" }
                                                li { "Click 'Create New Application'" }
                                                li { "Set Application Name: 'Pixelfed Rust Client'" }
                                                li { "Set Redirect URI: 'http://localhost:8080/callback'" }
                                                li { "Select Scopes: 'read' (and 'write' if needed)" }
                                                li { "Copy the Client ID and Client Secret here" }
                                            }
                                            p { 
                                                style: "margin-top: 10px; font-style: italic;",
                                                "Note: OAuth requires the instance to have OAUTH_ENABLED=true in their configuration."
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! {
                                    div {
                                        class: "public-api-info",
                                        h4 { "Public API Access" }
                                        p { "Using public API endpoints that don't require authentication." }
                                        ul {
                                            li { "✅ Public timelines and hashtags" }
                                            li { "✅ Public user profiles and posts" }
                                            li { "❌ Private content" }
                                            li { "❌ Higher rate limits" }
                                            li { "❌ Posting or interactions" }
                                        }
                                        p { 
                                            style: "margin-top: 10px; color: var(--text-secondary);",
                                            "Switch to OAuth for full access and authentication features."
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "download" => rsx! {
                        div {
                            class: "settings-section-content",
                            h3 { "📁 Download Settings" }
                            
                            div {
                                class: "form-group",
                                label { "Download Location:" }
                                div {
                                    class: "path-input",
                                    input {
                                        r#type: "text",
                                        value: "{temp_settings.current().download.base_path}",
                                        readonly: true,
                                    }
                                    button {
                                        class: "browse-btn",
                                        onclick: move |_| {
                                            // TODO: Implement folder picker
                                        },
                                        "📂 Browse"
                                    }
                                }
                                small { "Downloads will be saved to: {temp_settings.current().download.base_path}/pixelfed/" }
                            }

                            div {
                                class: "form-group",
                                label { "Max concurrent downloads:" }
                                input {
                                    r#type: "number",
                                    value: "{temp_settings.current().download.max_concurrent}",
                                    min: "1",
                                    max: "10",
                                    oninput: move |evt| {
                                        if let Ok(val) = evt.value.parse::<u32>() {
                                            let mut settings = temp_settings.current().as_ref().clone();
                                            settings.download.max_concurrent = val;
                                            temp_settings.set(settings);
                                        }
                                    },
                                }
                                small { "Number of files to download simultaneously (1-10)" }
                            }

                            div {
                                class: "form-group",
                                label { "Organize by date:" }
                                input {
                                    r#type: "checkbox",
                                    checked: temp_settings.current().download.organize_by_date,
                                    onchange: move |evt| {
                                        let mut settings = temp_settings.current().as_ref().clone();
                                        settings.download.organize_by_date = evt.value.parse().unwrap_or(true);
                                        temp_settings.set(settings);
                                    },
                                }
                                small { "Create folders with date stamps (username_2025-10-25)" }
                            }
                        }
                    },
                    _ => rsx! { div { "Unknown section" } }
                }

                // Save button at bottom
                div {
                    class: "settings-actions",
                    button {
                        class: "save-btn primary",
                        onclick: save_settings,
                        "💾 Save Settings"
                    }
                }
            }
        }
    })
}
