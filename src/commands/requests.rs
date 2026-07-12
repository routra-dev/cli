use anyhow::Result;

use crate::client::RoutraClient;
use super::CmdCtx;

pub async fn run(ctx: &CmdCtx, limit: u32, offset: u32) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    let resp = client
        .get(&format!("/requests?limit={}&offset={}", limit, offset))
        .await?;
    let data: serde_json::Value = resp.json().await?;

    if ctx.is_json() {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let items = data.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if items.is_empty() {
        println!("No recent requests.");
        return Ok(());
    }

    println!(
        "{:<36}  {:<20}  {:<14}  {:>8}  {:>10}  CREATED",
        "ID", "MODEL", "PROVIDER", "LATENCY", "COST (USD)"
    );
    for r in items {
        let id = r["id"].as_str().unwrap_or("-");
        let model = r["model"].as_str().unwrap_or("-");
        let provider = r["provider"].as_str().unwrap_or("-");
        let latency = r["latency_ms"].as_u64().unwrap_or(0);
        let cost = r["cost_usd"].as_f64().unwrap_or(0.0);
        let created = r["created_at"].as_str().unwrap_or("-");
        // Truncate long model names on a char boundary (a byte slice would
        // panic on multibyte names).
        let model_display: String = if model.chars().count() > 20 {
            model.chars().take(17).collect::<String>() + "..."
        } else {
            model.to_string()
        };
        println!(
            "{:<36}  {:<20}  {:<14}  {:>6}ms  {:>10}  {}",
            id,
            model_display,
            provider,
            latency,
            format!("${:.6}", cost),
            &created[..19.min(created.len())],
        );
    }

    Ok(())
}
