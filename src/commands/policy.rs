use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use serde::Serialize;

use crate::client::RoutraClient;
use super::CmdCtx;

#[derive(Subcommand)]
pub enum PolicyCmd {
    /// Push a routra.yaml policy file to your workspace
    Push {
        /// Path to policy YAML file
        file: String,
    },
    /// List policies in your workspace
    List,
}

pub async fn run(
    cmd: PolicyCmd,
    ctx: &CmdCtx,
) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    match cmd {
        PolicyCmd::Push { file } => {
            let contents =
                std::fs::read_to_string(&file).with_context(|| format!("reading {file}"))?;
            let parsed: serde_yml::Value =
                serde_yml::from_str(&contents).with_context(|| format!("parsing {file}"))?;

            // The server expects POST /v1/policies with {name, strategy, constraints}.
            // A policy YAML file may contain one or more policies under a "policies" key,
            // or be a single policy object at the top level.
            #[derive(Serialize)]
            struct CreatePolicy {
                name: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                strategy: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                constraints: Option<serde_json::Value>,
            }

            let policies_map = if let Some(mapping) = parsed["policies"].as_mapping() {
                // Multi-policy file: policies:\n  cheapest:\n    strategy: cheapest\n  ...
                mapping
                    .iter()
                    .map(|(k, v)| {
                        let name = k.as_str().unwrap_or("").to_string();
                        let strategy = v["strategy"].as_str().map(|s| s.to_string());
                        let constraints = v.get("constraints").and_then(|c| {
                            serde_json::to_value(c).ok()
                        });
                        CreatePolicy { name, strategy, constraints }
                    })
                    .collect::<Vec<_>>()
            } else if parsed["name"].as_str().is_some() {
                // Single policy object at top level
                let name = parsed["name"].as_str().unwrap_or("").to_string();
                let strategy = parsed["strategy"].as_str().map(|s| s.to_string());
                let constraints = parsed.get("constraints").and_then(|c| {
                    serde_json::to_value(c).ok()
                });
                vec![CreatePolicy { name, strategy, constraints }]
            } else {
                anyhow::bail!(
                    "Invalid policy file format. Expected a 'policies' mapping or a single policy with 'name' field."
                );
            };

            if policies_map.is_empty() {
                anyhow::bail!("No policies found in {file}");
            }

            for policy in &policies_map {
                client.post("/policies", policy).await?;
            }

            println!(
                "{} Pushed {} policy/policies from {}",
                "OK".green().bold(),
                policies_map.len(),
                file
            );
        }

        PolicyCmd::List => {
            let resp = client.get("/policies").await?;
            if ctx.is_json() {
                let data: serde_json::Value = resp.json().await?;
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }
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
    }

    Ok(())
}
