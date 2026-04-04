use anyhow::Result;

use crate::client::RoutraClient;

pub async fn run(model: Option<String>, days: u32, api_key: &Option<String>, base_url: &Option<String>) -> Result<()> {
    let client = RoutraClient::new(api_key, base_url)?;

    let mut path = format!("/usage?days={}", days);
    if let Some(m) = &model {
        path.push_str(&format!("&model={}", m));
    }

    let resp = client.get(&path).await?;
    let data: serde_json::Value = resp.json().await?;
    println!("{}", serde_json::to_string_pretty(&data)?);
    Ok(())
}
