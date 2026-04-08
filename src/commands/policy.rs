use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;

use crate::client::RoutraClient;

#[derive(Subcommand)]
pub enum PolicyCmd {
    /// Push a routra.yaml policy file to your workspace
    Push {
        /// Path to policy YAML file
        file: String,
    },
    /// List policies in your workspace
    List,
    /// Show details of a policy
    Get {
        /// Policy name or ID
        name: String,
    },
    /// Delete a policy
    Delete {
        /// Policy name or ID
        name: String,
    },
}

pub async fn run(
    cmd: PolicyCmd,
    api_key: &Option<String>,
    base_url: &Option<String>,
) -> Result<()> {
    let client = RoutraClient::new(api_key, base_url)?;

    match cmd {
        PolicyCmd::Push { file } => {
            let contents =
                std::fs::read_to_string(&file).with_context(|| format!("reading {file}"))?;
            let parsed: serde_yaml::Value =
                serde_yaml::from_str(&contents).with_context(|| format!("parsing {file}"))?;

            #[derive(serde::Serialize)]
            struct Req {
                yaml: String,
            }
            client
                .post("/policies/push", &Req { yaml: contents })
                .await?;
            let policy_count = parsed["policies"]
                .as_mapping()
                .map(|m| m.len())
                .unwrap_or(0);
            println!(
                "{} Pushed {} policy/policies from {}",
                "OK".green().bold(),
                policy_count,
                file
            );
        }

        PolicyCmd::List => {
            let resp = client.get("/policies").await?;
            let list: Vec<serde_json::Value> = resp.json().await?;
            if list.is_empty() {
                println!("No policies. Push one with `routra policy push routra.yaml`");
                return Ok(());
            }
            println!("{:<36}  {:<20}  STRATEGY", "ID", "NAME");
            for p in list {
                println!(
                    "{:<36}  {:<20}  {}",
                    p["id"].as_str().unwrap_or(""),
                    p["name"].as_str().unwrap_or(""),
                    p["strategy"].as_str().unwrap_or(""),
                );
            }
        }

        PolicyCmd::Get { name } => {
            let resp = client.get(&format!("/policies/{}", name)).await?;
            let p: serde_json::Value = resp.json().await?;
            println!("{}", serde_json::to_string_pretty(&p)?);
        }

        PolicyCmd::Delete { name } => {
            client.delete(&format!("/policies/{}", name)).await?;
            println!("{} Policy {} deleted.", "OK".green().bold(), name);
        }
    }

    Ok(())
}
