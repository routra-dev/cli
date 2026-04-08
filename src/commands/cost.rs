use anyhow::Result;

use crate::client::RoutraClient;

pub async fn run(
    breakdown: String,
    days: u32,
    api_key: &Option<String>,
    base_url: &Option<String>,
) -> Result<()> {
    let client = RoutraClient::new(api_key, base_url)?;

    let path = format!("/usage/cost?days={}&breakdown={}", days, breakdown);
    let resp = client.get(&path).await?;
    let data: serde_json::Value = resp.json().await?;

    let items = match data.as_array() {
        Some(arr) if !arr.is_empty() => arr,
        _ => {
            println!("No cost data for the selected period.");
            return Ok(());
        }
    };

    // Header
    println!(
        "{:<24} {:<16} {:>12} {:>10}",
        "Model", "Provider", "Requests", "Cost (USD)"
    );
    println!("{:-<66}", "");
    for item in items {
        let model = item["model"].as_str().unwrap_or("-");
        let provider = item["provider"].as_str().unwrap_or("-");
        let count = item["request_count"].as_i64().unwrap_or(0);
        let cost = item["total_cost_usd"].as_f64().unwrap_or(0.0);
        println!(
            "{:<24} {:<16} {:>12} {:>10}",
            model,
            provider,
            count,
            format!("${:.6}", cost)
        );
    }

    Ok(())
}
