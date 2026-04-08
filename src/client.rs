use anyhow::{bail, Result};
use reqwest::{header, Client, Response};
use serde::Serialize;

use crate::config;

const DEFAULT_BASE_URL: &str = "https://api.routra.dev/v1";

pub struct RoutraClient {
    inner: Client,
    api_key: String,
    pub base_url: String,
}

impl RoutraClient {
    pub fn new(api_key_override: &Option<String>, base_url_override: &Option<String>) -> Result<Self> {
        let cfg = config::load().unwrap_or_default();

        let api_key = api_key_override
            .clone()
            .or(cfg.api_key)
            .ok_or_else(|| anyhow::anyhow!(
                "No API key found. Run `routra login` or set ROUTRA_API_KEY."
            ))?;

        let base_url = base_url_override
            .clone()
            .or(cfg.base_url)
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());

        Ok(Self {
            inner: Client::new(),
            api_key,
            base_url,
        })
    }

    pub async fn get(&self, path: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.inner
            .get(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await?;
        check_status(resp).await
    }

    pub async fn post<B: Serialize>(&self, path: &str, body: &B) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.inner
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(body)
            .send()
            .await?;
        check_status(resp).await
    }

    /// POST with no body. Does NOT check status — caller handles the response.
    pub async fn post_empty(&self, path: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.inner
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await?;
        Ok(resp)
    }

    pub async fn delete(&self, path: &str) -> Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.inner
            .delete(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .send()
            .await?;
        check_status(resp).await
    }
}

async fn check_status(resp: Response) -> Result<Response> {
    if resp.status().is_success() {
        Ok(resp)
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("API error {}: {}", status, body)
    }
}
