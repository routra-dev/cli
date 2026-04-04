use anyhow::Result;

use crate::client::RoutraClient;

pub async fn run(breakdown: String, days: u32, api_key: &Option<String>, base_url: &Option<String>) -> Result<()> {
    let client = RoutraClient::new(api_key, base_url)?;

    let path = format!("/usage/cost?days={}&breakdown={}", days, breakdown);
    let resp = client.get(&path).await?;
    let data: serde_json::Value = resp.json().await?;
    println!("{}", serde_json::to_string_pretty(&data)?);
    Ok(())
}
