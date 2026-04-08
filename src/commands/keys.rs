use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use serde::Deserialize;

use crate::client::RoutraClient;

#[derive(Subcommand)]
pub enum KeysCmd {
    /// List all API keys
    List,
    /// Create a new API key
    Create {
        /// Human-readable name for the key
        #[arg(long)]
        name: String,
        /// Attach a routing policy to this key
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
#[allow(dead_code)]
struct ApiKey {
    id: String,
    name: String,
    prefix: String,
    is_active: bool,
    created_at: String,
    last_used_at: Option<String>,
}

pub async fn run(cmd: KeysCmd, api_key: &Option<String>, base_url: &Option<String>) -> Result<()> {
    let client = RoutraClient::new(api_key, base_url)?;

    match cmd {
        KeysCmd::List => {
            let resp = client.get("/keys").await?;
            let keys: Vec<ApiKey> = resp.json().await?;

            if keys.is_empty() {
                println!("No API keys found. Create one with `routra keys create --name <name>`");
                return Ok(());
            }

            println!("{:<36}  {:<20}  {:<12}  STATUS", "ID", "NAME", "PREFIX");
            for k in keys {
                let status = if k.is_active {
                    "active".green()
                } else {
                    "revoked".red()
                };
                println!("{:<36}  {:<20}  {:<12}  {}", k.id, k.name, k.prefix, status);
            }
        }

        KeysCmd::Create { name, policy } => {
            #[derive(serde::Serialize)]
            struct Req {
                name: String,
                policy_name: Option<String>,
            }
            let resp = client
                .post(
                    "/keys",
                    &Req {
                        name,
                        policy_name: policy,
                    },
                )
                .await?;
            let key: serde_json::Value = resp.json().await?;
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
            client.delete(&format!("/keys/{}", id)).await?;
            println!("{} Key {} revoked.", "OK".green().bold(), id);
        }
    }

    Ok(())
}
