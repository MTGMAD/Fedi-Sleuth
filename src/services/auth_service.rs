use anyhow::Result;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenUrl,
};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

use crate::models::ApiSettings;

pub struct AuthService {
    client: Option<BasicClient>,
    http_client: Client,
    settings: ApiSettings,
    instance_url: String,
    redirect_uri: String,
}

#[allow(dead_code)]
impl AuthService {
    #[allow(dead_code)]
    pub fn new(settings: ApiSettings, instance_url: &str) -> Result<Self> {
        Self::new_with_redirect(settings, instance_url, "http://localhost:8080/callback")
    }

    pub fn new_with_redirect(settings: ApiSettings, instance_url: &str, redirect_uri: &str) -> Result<Self> {
        let client = if settings.use_oauth && !settings.client_id.is_empty() {
            let auth_url = AuthUrl::new(format!("{}/oauth/authorize", instance_url))?;
            let token_url = TokenUrl::new(format!("{}/oauth/token", instance_url))?;

            let client = BasicClient::new(
                ClientId::new(settings.client_id.clone()),
                Some(ClientSecret::new(settings.client_secret.clone())),
                auth_url,
                Some(token_url),
            )
            .set_auth_type(AuthType::RequestBody)
            .set_redirect_uri(RedirectUrl::new(redirect_uri.to_string())?);

            Some(client)
        } else {
            None
        };

        Ok(Self {
            client,
            http_client: Client::new(),
            settings,
            instance_url: instance_url.to_string(),
            redirect_uri: redirect_uri.to_string(),
        })
    }

    /// Register a new OAuth application with the Pixelfed instance
    pub async fn register_app(&self, app_name: &str) -> Result<(String, String)> {
        let url = format!("{}/api/v1/apps", self.instance_url);

        let mut params = HashMap::new();
        params.insert("client_name", app_name);
        params.insert("redirect_uris", self.redirect_uri.as_str());
        params.insert("scopes", "read write");
        params.insert("website", "https://github.com/pixelfed/rust-client");

        let response = self
            .http_client
            .post(&url)
            .form(&params)
            .header("User-Agent", "PixelfedRustClient/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to register app: {}",
                response.status()
            ));
        }

        let app_data: Value = response.json().await?;

        let client_id = app_data["client_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing client_id in response"))?
            .to_string();

        let client_secret = app_data["client_secret"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing client_secret in response"))?
            .to_string();

        Ok((client_id, client_secret))
    }

    /// Check if the instance supports OAuth
    #[allow(dead_code)]
    pub async fn check_oauth_support(&self) -> Result<bool> {
        let url = format!("{}/api/v1/instance", self.instance_url);

        let response = self
            .http_client
            .get(&url)
            .header("User-Agent", "PixelfedRustClient/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(false); // Assume no OAuth support if instance endpoint fails
        }

        // Most Pixelfed instances have OAuth enabled by default
        Ok(true)
    }

    /// Verify if an access token is valid
    pub async fn verify_token(&self, access_token: &str) -> Result<Value> {
        let url = format!("{}/api/v1/accounts/verify_credentials", self.instance_url);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "PixelfedRustClient/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Token verification failed: {}",
                response.status()
            ));
        }

        let user_data: Value = response.json().await?;
        Ok(user_data)
    }

    pub fn generate_auth_url(&self) -> Result<(Url, CsrfToken)> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("OAuth not configured"))?;

        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("read".to_string()))
            .add_scope(Scope::new("write".to_string()))
            .url();

        Ok((auth_url, csrf_token))
    }

    pub async fn exchange_code(
        &self,
        code: AuthorizationCode,
        _csrf_token: CsrfToken,
    ) -> Result<String> {
        if self.client.is_none() {
            return Err(anyhow::anyhow!("OAuth not configured"));
        }

        let token_url = format!("{}/oauth/token", self.instance_url);

        let mut params = HashMap::new();
        params.insert(
            "grant_type".to_string(),
            "authorization_code".to_string(),
        );
        params.insert("code".to_string(), code.secret().to_string());
        params.insert(
            "redirect_uri".to_string(),
            self.redirect_uri.clone(),
        );
        params.insert("client_id".to_string(), self.settings.client_id.clone());
        params.insert(
            "client_secret".to_string(),
            self.settings.client_secret.clone(),
        );

        let response = self
            .http_client
            .post(&token_url)
            .form(&params)
            .header("User-Agent", "PixelfedRustClient/1.0")
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Token exchange failed: {} - {}",
                status,
                body
            ));
        }

        let token_data: Value = response.json().await?;
        let access_token = token_data
            .get("access_token")
            .and_then(|value| value.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing access_token in token response"))?;

        Ok(access_token.to_string())
    }

    /// Revoke an access token
    pub async fn revoke_token(&self, access_token: &str) -> Result<()> {
        let url = format!("{}/oauth/revoke", self.instance_url);

        let mut params = HashMap::new();
        params.insert("token", access_token);
        if !self.settings.client_id.is_empty() {
            params.insert("client_id", &self.settings.client_id);
        }
        if !self.settings.client_secret.is_empty() {
            params.insert("client_secret", &self.settings.client_secret);
        }

        let response = self
            .http_client
            .post(&url)
            .form(&params)
            .header("User-Agent", "PixelfedRustClient/1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to revoke token: {}",
                response.status()
            ));
        }

        Ok(())
    }

    pub fn is_authenticated(&self) -> bool {
        self.settings.access_token.is_some()
    }

    pub fn get_access_token(&self) -> Option<&str> {
        self.settings.access_token.as_deref()
    }

    pub fn is_using_oauth(&self) -> bool {
        self.settings.use_oauth
    }

    pub fn supports_public_api(&self) -> bool {
        !self.settings.use_oauth
    }
}
