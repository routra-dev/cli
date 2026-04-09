use anyhow::Result;

use crate::client::RoutraClient;

pub async fn run(
    api_key: &Option<String>,
    base_url: &Option<String>,
) -> Result<()> {
    let client = RoutraClient::new(api_key, base_url)?;

    let resp = client.get("/usage").await?;
    let data: serde_json::Value = resp.json().await?;

    let period = data["period"].as_str().unwrap_or("-");
    let requests = data["total_requests"].as_i64().unwrap_or(0);
    let input_tok = data["total_input_tokens"].as_i64().unwrap_or(0);
    let output_tok = data["total_output_tokens"].as_i64().unwrap_or(0);
    let cost = data["total_cost_usd"].as_f64().unwrap_or(0.0);

    println!("Usage Summary ({})", period);
    println!("{:-<40}", "");
    println!("{:<22} {:>12}", "Requests", requests);
    println!("{:<22} {:>12}", "Input tokens", input_tok);
    println!("{:<22} {:>12}", "Output tokens", output_tok);
    println!("{:<22} {:>12}", "Spend (MTD)", format!("${:.6}", cost));

    Ok(())
}
