use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;

use crate::client::RoutraClient;
use super::CmdCtx;

#[derive(Subcommand)]
pub enum BatchCmd {
    /// Submit a JSONL file as a batch job
    Create {
        /// Path to JSONL file with one chat request per line
        file: String,
        /// Policy to apply to this batch
        #[arg(long)]
        policy: Option<String>,
        /// Completion window (e.g. "24h", "1h", "30m")
        #[arg(long, default_value = "24h")]
        window: String,
    },
    /// Check batch job status
    Status {
        /// Batch job ID
        id: String,
    },
    /// Get results download URL for a completed batch
    Results {
        /// Batch job ID
        id: String,
    },
    /// Cancel a queued or processing batch job
    Cancel {
        /// Batch job ID
        id: String,
    },
    /// List all batch jobs
    List,
}

pub async fn run(cmd: BatchCmd, ctx: &CmdCtx) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    match cmd {
        BatchCmd::Create {
            file,
            policy,
            window,
        } => {
            let contents =
                std::fs::read_to_string(&file).with_context(|| format!("reading {file}"))?;

            // Parse JSONL → Vec<serde_json::Value> to match server schema
            let mut requests: Vec<serde_json::Value> = Vec::new();
            for (i, line) in contents.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let item: serde_json::Value = serde_json::from_str(line)
                    .with_context(|| format!("invalid JSON on line {}", i + 1))?;
                requests.push(item);
            }

            if requests.is_empty() {
                anyhow::bail!("JSONL file is empty — no requests to submit");
            }

            let line_count = requests.len();

            #[derive(serde::Serialize)]
            struct Req {
                requests: Vec<serde_json::Value>,
                completion_window: String,
                #[serde(skip_serializing_if = "Option::is_none")]
                policy_name: Option<String>,
            }
            let resp = client
                .post(
                    "/batch",
                    &Req {
                        requests,
                        completion_window: window,
                        policy_name: policy,
                    },
                )
                .await?;
            let job: serde_json::Value = resp.json().await?;

            println!(
                "{} Batch job submitted ({} requests).",
                "OK".green().bold(),
                line_count
            );
            println!("  ID: {}", job["id"].as_str().unwrap_or("").bold());
            println!(
                "  Poll with: routra batch status {}",
                job["id"].as_str().unwrap_or("")
            );
        }

        BatchCmd::Status { id } => {
            let resp = client.get(&format!("/batch/{}/status", id)).await?;
            let job: serde_json::Value = resp.json().await?;
            let status = job["status"].as_str().unwrap_or("unknown");
            let completed = job["completed_count"].as_u64().unwrap_or(0);
            let total = job["request_count"].as_u64().unwrap_or(0);
            let pct = if total > 0 {
                completed * 100 / total
            } else {
                0
            };

            let status_colored = match status {
                "complete" => status.green().bold(),
                "failed" => status.red().bold(),
                "cancelled" => status.red().bold(),
                "processing" => status.yellow().bold(),
                _ => status.normal(),
            };

            println!(
                "Status: {} ({}/{} = {}%)",
                status_colored, completed, total, pct
            );
            if let Some(cost) = job["cost_usd"].as_f64() {
                println!("Cost:   ${:.6}", cost);
            }
            if let Some(err) = job["error_message"].as_str() {
                println!("Error:  {}", err.red());
            }
        }

        BatchCmd::Results { id } => {
            let resp = client.get(&format!("/batch/{}/results", id)).await?;
            let data: serde_json::Value = resp.json().await?;
            if let Some(url) = data["results_url"].as_str() {
                println!("Results URL (valid 24h): {}", url.bold());
            } else if data["results"].is_array() {
                // DB fallback - results are inline
                println!("{}", serde_json::to_string_pretty(&data["results"])?);
            } else {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }

        BatchCmd::Cancel { id } => {
            client.post_empty(&format!("/batch/{}/cancel", id)).await?;
            println!("{} Batch job {} cancelled.", "OK".green().bold(), id);
        }

        BatchCmd::List => {
            let resp = client.get("/batch").await?;
            let jobs: Vec<serde_json::Value> = resp.json().await?;
            if jobs.is_empty() {
                println!("No batch jobs found.");
                return Ok(());
            }
            println!(
                "{:<36}  {:<12}  {:<8}  COST USD",
                "ID", "STATUS", "REQUESTS"
            );
            for j in jobs {
                println!(
                    "{:<36}  {:<12}  {:<8}  ${:.6}",
                    j["id"].as_str().unwrap_or(""),
                    j["status"].as_str().unwrap_or(""),
                    j["request_count"].as_u64().unwrap_or(0),
                    j["cost_usd"].as_f64().unwrap_or(0.0),
                );
            }
        }
    }

    Ok(())
}
