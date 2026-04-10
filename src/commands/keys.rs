use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use serde::Deserialize;

use crate::client::RoutraClient;
use super::CmdCtx;

#[derive(Subcommand)]
pub enum KeysCmd {
    /// List all API keys
    List,
    /// Create a new API key
    Create {
        /// Human-readable name for the key
        #[arg(long)]
        name: String,
        /// Attach a routing policy (ID) to this key
        #[arg(long)]
        policy: Option<String>,
    },
    /// Rotate an existing key (old key stays active 24h)
    Rotate {
        /// Key ID to rotate
        id: String,
    },
    /// Revoke an API key
    Revoke {
        /// Key ID to revoke
        id: String,
    },
}

#[derive(Deserialize)]
struct ApiKey {
    id: String,
    name: String,
    prefix: String,
    is_active: bool,
    _created_at: String,
    last_used_at: Option<String>,
}

pub async fn run(cmd: KeysCmd, ctx: &CmdCtx) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    match cmd {
        KeysCmd::List => {
            let resp = client.get("/keys").await?;
            if ctx.is_json() {
                let data: serde_json::Value = resp.json().await?;
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }
            let keys: Vec<ApiKey> = resp.json().await?;

            if keys.is_empty() {
                println!("No API keys found. Create one with `routra keys create --name <name>`");
                return Ok(());
            }

            println!("{:<36}  {:<20}  {:<12}  {:<8}  LAST USED", "ID", "NAME", "PREFIX", "STATUS");
            for k in keys {
                let status = if k.is_active {
                    "active".green()
                } else {
                    "revoked".red()
                };
                let last_used = k.last_used_at.as_deref().unwrap_or("never");
                println!("{:<36}  {:<20}  {:<12}  {:<8}  {}", k.id, k.name, k.prefix, status, last_used);
            }
        }

        KeysCmd::Create { name, policy } => {
            #[derive(serde::Serialize)]
            struct Req {
                name: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                policy_id: Option<String>,
            }
            let resp = client
                .post(
                    "/keys",
                    &Req {
                        name,
                        policy_id: policy,
                    },
                )
                .await?;
            let key: serde_json::Value = resp.json().await?;
            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&key)?);
                return Ok(());
            }
            println!("{} API key created.", "OK".green().bold());
            println!();
            println!(
                "  Key (shown once): {}",
                key["key"].as_str().unwrap_or("").bold()
            );
            println!("  ID:               {}", key["id"].as_str().unwrap_or(""));
            println!();
            println!(
                "{}",
                "Store this key - it will not be shown again.".yellow()
            );
        }

        KeysCmd::Rotate { id } => {
            #[derive(serde::Serialize)]
            struct Req {}
            let resp = client
                .post(&format!("/keys/{}/rotate", id), &Req {})
                .await?;
            let key: serde_json::Value = resp.json().await?;
            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&key)?);
                return Ok(());
            }
            println!(
                "{} Key rotated. Old key active for 24h.",
                "OK".green().bold()
            );
            println!();
            println!(
                "  New key (shown once): {}",
                key["key"].as_str().unwrap_or("").bold()
            );
            println!();
            println!(
                "{}",
                "Store this key - it will not be shown again.".yellow()
            );
        }

        KeysCmd::Revoke { id } => {
            if !ctx.confirm(&format!("Revoke key {}? This cannot be undone.", id)) {
                println!("Cancelled.");
                return Ok(());
            }
            client.delete(&format!("/keys/{}", id)).await?;
            if ctx.is_json() {
                println!(r#"{{"ok": true, "id": "{}"}}"#, id);
            } else {
                println!("{} Key {} revoked.", "OK".green().bold(), id);
            }
        }
    }

    Ok(())
}
