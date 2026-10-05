use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde_json::{Value, json};
use uuid::Uuid;
#[allow(dead_code)]
pub const BASE: &str = "http://127.0.0.1:8080";

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestClient {
    pub http: Client,
    pub access: String,
    pub refresh: String,
}

#[allow(dead_code)]
impl TestClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
            access: String::new(),
            refresh: String::new(),
        }
    }

    pub fn with_tokens(access: String, refresh: String) -> Self {
        Self {
            http: Client::new(),
            access,
            refresh,
        }
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<(StatusCode, Value)> {
        let res = self
            .http
            .post(format!("{BASE}{path}"))
            .header("Authorization", format!("Bearer {}", self.access))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        let status = res.status();
        let v: Value = res.json().await.unwrap_or(json!({}));
        Ok((status, v))
    }

    pub async fn get(&self, path: &str) -> Result<(StatusCode, Value)> {
        let res = self
            .http
            .get(format!("{BASE}{path}"))
            .header("Authorization", format!("Bearer {}", self.access))
            .send()
            .await?;
        let status = res.status();
        let v: Value = res.json().await.unwrap_or(json!({}));
        Ok((status, v))
    }

    pub async fn patch(&self, path: &str, body: Value) -> Result<(StatusCode, Value)> {
        let res = self
            .http
            .patch(format!("{BASE}{path}"))
            .header("Authorization", format!("Bearer {}", self.access))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;
        let status = res.status();
        let v: Value = res.json().await.unwrap_or(json!({}));
        Ok((status, v))
    }
}

#[allow(dead_code)]
pub fn extract_tokens(resp: &Value) -> Result<(String, String)> {
    let meta = resp.get("meta").context("missing meta")?;
    let access = meta["access_token"]
        .as_str()
        .context("missing access_token")?
        .to_string();
    let refresh = meta["refresh_token"]
        .as_str()
        .context("missing refresh_token")?
        .to_string();
    Ok((access, refresh))
}

#[allow(dead_code)]
pub fn extract_uuid(v: &Value, path: &[&str]) -> Result<Uuid> {
    let mut cur = v;
    for key in path {
        cur = cur
            .get(*key)
            .with_context(|| format!("missing key '{key}' in response"))?;
    }
    let s = cur.as_str().context("expected string uuid")?;
    Ok(s.parse()?)
}
