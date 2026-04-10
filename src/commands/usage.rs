use anyhow::Result;

use crate::client::RoutraClient;
use super::CmdCtx;

pub async fn run(ctx: &CmdCtx) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    let resp = client.get("/usage").await?;
    let data: serde_json::Value = resp.json().await?;

    if ctx.is_json() {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let period = data["period_start"].as_str().unwrap_or("-");
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

    // Modality breakdown
    if let Some(mods) = data["modality_breakdown"].as_array() {
        if !mods.is_empty() {
            println!();
            println!("By Modality:");
            println!("{:<18} {:>10} {:>12}", "TYPE", "REQUESTS", "COST (USD)");
            for m in mods {
                let unit = m["usage_unit"].as_str().unwrap_or("-");
                let count = m["request_count"].as_i64().unwrap_or(0);
                let unit_cost = m["total_cost_usd"].as_f64().unwrap_or(0.0);
                let label = match unit {
                    "tokens" => "Chat/Embeddings",
                    "images" => "Image Gen",
                    "characters" => "TTS",
                    "seconds" => "STT",
                    "steps" => "Image (steps)",
                    other => other,
                };
                println!("{:<18} {:>10} {:>12}", label, count, format!("${:.6}", unit_cost));
            }
        }
    }

    Ok(())
}
