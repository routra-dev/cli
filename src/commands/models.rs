use anyhow::Result;

use crate::client::RoutraClient;
use super::CmdCtx;

pub async fn run(ctx: &CmdCtx) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    let resp = client.get("/models/catalog").await?;
    let data: serde_json::Value = resp.json().await?;

    if ctx.is_json() {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let items = data.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if items.is_empty() {
        println!("No models found.");
        return Ok(());
    }

    println!(
        "{:<36}  {:<10}  {:<12}  {:<10}  CAPABILITIES",
        "SLUG", "TIER", "TYPE", "PARAMS(B)"
    );
    for m in items {
        let slug = m["slug"].as_str().unwrap_or("-");
        let tier = m["tier"].as_str().unwrap_or("-");
        let model_type = m["model_type"].as_str().unwrap_or("-");
        let params = m["param_count_b"]
            .as_f64()
            .map(|v| format!("{:.0}", v))
            .unwrap_or_else(|| "-".to_string());
        let caps = m["capabilities"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter(|(_, v)| v.as_bool().unwrap_or(false))
                    .map(|(k, _)| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<36}  {:<10}  {:<12}  {:<10}  {}",
            slug, tier, model_type, params, caps
        );
    }

    Ok(())
}
