use anyhow::Result;

use crate::client::RoutraClient;
use super::CmdCtx;

pub async fn run(ctx: &CmdCtx, limit: u32, offset: u32) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    let resp = client
        .get(&format!("/audit-log?limit={}&offset={}", limit, offset))
        .await?;
    let data: serde_json::Value = resp.json().await?;

    if ctx.is_json() {
        println!("{}", serde_json::to_string_pretty(&data)?);
        return Ok(());
    }

    let items = data.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
    if items.is_empty() {
        println!("No audit log entries.");
        return Ok(());
    }

    println!(
        "{:<36}  {:<20}  {:<16}  {:<36}  CREATED",
        "ID", "ACTION", "RESOURCE TYPE", "RESOURCE ID"
    );
    for e in items {
        let id = e["id"].as_str().unwrap_or("-");
        let action = e["action"].as_str().unwrap_or("-");
        let res_type = e["resource_type"].as_str().unwrap_or("-");
        let res_id = e["resource_id"].as_str().unwrap_or("-");
        let created = e["created_at"].as_str().unwrap_or("-");
        println!(
            "{:<36}  {:<20}  {:<16}  {:<36}  {}",
            id,
            action,
            res_type,
            res_id,
            &created[..19.min(created.len())],
        );
    }

    Ok(())
}
