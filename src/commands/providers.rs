use anyhow::Result;

use crate::client::RoutraClient;
use super::CmdCtx;

pub async fn run(ctx: &CmdCtx) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    let resp = client.get("/providers").await?;
    let data: serde_json::Value = resp.json().await?;

    if ctx.is_json() {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let items = data.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if items.is_empty() {
        println!("No providers found.");
        return Ok(());
    }

    println!("{:<20}  {:<24}  {:<8}  MODALITIES", "SLUG", "NAME", "HEALTHY");
    for p in items {
        let slug = p["slug"].as_str().unwrap_or("-");
        let name = p["name"].as_str().unwrap_or("-");
        let healthy = if p["is_healthy"].as_bool().unwrap_or(false) {
            "yes"
        } else {
            "no"
        };
        let modalities = p["supported_modalities"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "-".to_string());
        println!("{:<20}  {:<24}  {:<8}  {}", slug, name, healthy, modalities);
    }

    Ok(())
}
