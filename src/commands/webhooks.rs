use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::client::RoutraClient;
use super::CmdCtx;

#[derive(Subcommand)]
pub enum WebhooksCmd {
    /// List webhook endpoints
    List,
    /// Create a new webhook endpoint
    Create {
        /// URL to receive webhook events (must be HTTPS)
        #[arg(long)]
        url: String,
        /// Events to subscribe to (comma-separated, e.g. "spend.cap,key.rotated")
        #[arg(long, value_delimiter = ',')]
        events: Vec<String>,
    },
    /// Delete a webhook endpoint
    Delete {
        /// Webhook ID to delete
        id: String,
    },
}

pub async fn run(cmd: WebhooksCmd, ctx: &CmdCtx) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    match cmd {
        WebhooksCmd::List => {
            let resp = client.get("/webhooks").await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            let webhooks = data.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
            if webhooks.is_empty() {
                println!("No webhook endpoints configured.");
                return Ok(());
            }

            println!("{:<36}  {:<40}  {:<8}  EVENTS", "ID", "URL", "ACTIVE");
            for w in webhooks {
                let id = w["id"].as_str().unwrap_or("");
                let url = w["url"].as_str().unwrap_or("");
                let active = w["active"].as_bool().unwrap_or(false);
                let events = w["events"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|e| e.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                let active_str = if active {
                    "yes".green().to_string()
                } else {
                    "no".red().to_string()
                };
                println!("{:<36}  {:<40}  {:<8}  {}", id, url, active_str, events);
            }
        }

        WebhooksCmd::Create { url, events } => {
            #[derive(serde::Serialize)]
            struct Req {
                url: String,
                events: Vec<String>,
            }
            let resp = client.post("/webhooks", &Req { url, events }).await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            println!("{} Webhook created.", "OK".green().bold());
            println!("  ID: {}", data["id"].as_str().unwrap_or("").bold());
        }

        WebhooksCmd::Delete { id } => {
            if !ctx.confirm(&format!("Delete webhook {}?", id)) {
                println!("Cancelled.");
                return Ok(());
            }
            client.delete(&format!("/webhooks/{}", id)).await?;
            if ctx.is_json() {
                println!(r#"{{"ok": true, "id": "{}"}}"#, id);
            } else {
                println!("{} Webhook {} deleted.", "OK".green().bold(), id);
            }
        }
    }

    Ok(())
}
