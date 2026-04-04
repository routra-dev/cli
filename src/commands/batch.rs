use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;

use crate::client::RoutraClient;

#[derive(Subcommand)]
pub enum BatchCmd {
    /// Submit a JSONL file as a batch job
    Create {
        /// Path to JSONL file with one chat request per line
        file: String,
        /// Policy to apply to this batch
        #[arg(long)]
        policy: Option<String>,
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
    /// List all batch jobs
    List,
}

pub async fn run(cmd: BatchCmd, api_key: &Option<String>, base_url: &Option<String>) -> Result<()> {
    let client = RoutraClient::new(api_key, base_url)?;

    match cmd {
        BatchCmd::Create { file, policy } => {
            let contents = std::fs::read_to_string(&file)
                .with_context(|| format!("reading {file}"))?;
            let line_count = contents.lines().count();

            #[derive(serde::Serialize)]
            struct Req { requests_jsonl: String, policy_name: Option<String> }
            let resp = client.post("/batch", &Req { requests_jsonl: contents, policy_name: policy }).await?;
            let job: serde_json::Value = resp.json().await?;

            println!("{} Batch job submitted ({} requests).", "OK".green().bold(), line_count);
            println!("  ID: {}", job["id"].as_str().unwrap_or("").bold());
            println!("  Poll with: routra batch status {}", job["id"].as_str().unwrap_or(""));
        }

        BatchCmd::Status { id } => {
            let resp = client.get(&format!("/batch/{}/status", id)).await?;
            let job: serde_json::Value = resp.json().await?;
            let status = job["status"].as_str().unwrap_or("unknown");
            let completed = job["completed_count"].as_u64().unwrap_or(0);
            let total = job["request_count"].as_u64().unwrap_or(0);
            let pct = if total > 0 { completed * 100 / total } else { 0 };

            let status_colored = match status {
                "complete" => status.green().bold(),
                "failed" => status.red().bold(),
                "running" => status.yellow().bold(),
                _ => status.normal(),
            };

            println!("Status: {} ({}/{} = {}%)", status_colored, completed, total, pct);
            if let Some(cost) = job["cost_usd"].as_f64() {
                println!("Cost:   ${:.6}", cost);
            }
        }

        BatchCmd::Results { id } => {
            let resp = client.get(&format!("/batch/{}/results", id)).await?;
            let data: serde_json::Value = resp.json().await?;
            if let Some(url) = data["url"].as_str() {
                println!("Results URL (valid 1h): {}", url.bold());
            } else {
                println!("{}", serde_json::to_string_pretty(&data)?);
            }
        }

        BatchCmd::List => {
            let resp = client.get("/batch").await?;
            let jobs: Vec<serde_json::Value> = resp.json().await?;
            if jobs.is_empty() {
                println!("No batch jobs found.");
                return Ok(());
            }
            println!("{:<36}  {:<10}  {:<8}  {}", "ID", "STATUS", "REQUESTS", "COST USD");
            for j in jobs {
                println!(
                    "{:<36}  {:<10}  {:<8}  ${:.6}",
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
