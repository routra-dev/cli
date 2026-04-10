use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::client::RoutraClient;
use super::CmdCtx;

#[derive(Subcommand)]
pub enum ProviderKeysCmd {
    /// List stored provider keys
    List,
    /// Store a provider key (BYOK)
    Store {
        /// Provider slug (e.g. openai, anthropic, together)
        provider: String,
        /// API key for the provider
        #[arg(long)]
        key: String,
    },
    /// Verify a stored provider key works
    Verify {
        /// Provider slug to verify
        provider: String,
    },
    /// Delete a stored provider key
    Delete {
        /// Provider slug to delete
        provider: String,
    },
}

pub async fn run(cmd: ProviderKeysCmd, ctx: &CmdCtx) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    match cmd {
        ProviderKeysCmd::List => {
            let resp = client.get("/provider-keys").await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            let keys = data.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
            if keys.is_empty() {
                println!("No provider keys stored. Add one with `routra provider-keys store <provider> --key <key>`");
                return Ok(());
            }

            println!("{:<20}  CREATED", "PROVIDER");
            for k in keys {
                println!(
                    "{:<20}  {}",
                    k["provider_slug"].as_str().unwrap_or("-"),
                    k["created_at"].as_str().unwrap_or("-"),
                );
            }
        }

        ProviderKeysCmd::Store { provider, key } => {
            #[derive(serde::Serialize)]
            struct Req {
                api_key: String,
            }
            client
                .post(&format!("/provider-keys/{}", provider), &Req { api_key: key })
                .await?;

            if ctx.is_json() {
                println!(r#"{{"ok": true, "provider": "{}"}}"#, provider);
            } else {
                println!(
                    "{} Provider key stored for {}.",
                    "OK".green().bold(),
                    provider.bold()
                );
            }
        }

        ProviderKeysCmd::Verify { provider } => {
            #[derive(serde::Serialize)]
            struct Req {}
            let resp = client
                .post(&format!("/provider-keys/{}/verify", provider), &Req {})
                .await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            let valid = data["valid"].as_bool().unwrap_or(false);
            if valid {
                println!("{} Key for {} is valid.", "OK".green().bold(), provider);
            } else {
                println!(
                    "{} Key for {} is invalid or expired.",
                    "FAIL".red().bold(),
                    provider
                );
            }
        }

        ProviderKeysCmd::Delete { provider } => {
            if !ctx.confirm(&format!("Delete stored key for {}?", provider)) {
                println!("Cancelled.");
                return Ok(());
            }
            client
                .delete(&format!("/provider-keys/{}", provider))
                .await?;

            if ctx.is_json() {
                println!(r#"{{"ok": true, "provider": "{}"}}"#, provider);
            } else {
                println!(
                    "{} Provider key for {} deleted.",
                    "OK".green().bold(),
                    provider
                );
            }
        }
    }

    Ok(())
}
