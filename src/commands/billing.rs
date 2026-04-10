use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;

use crate::client::RoutraClient;
use super::CmdCtx;

#[derive(Subcommand)]
pub enum BillingCmd {
    /// Show current billing info (tier, credit balance, spend)
    Info,
    /// Create a checkout for plan upgrade
    Checkout {
        /// Plan to upgrade to (pro, scale)
        plan: String,
        /// Success redirect URL (must be HTTPS)
        #[arg(long)]
        success_url: String,
    },
    /// Cancel current subscription
    Cancel,
    /// Top up credits
    Topup {
        /// Amount in USD ($5-$500)
        amount: f64,
        /// Success redirect URL (must be HTTPS)
        #[arg(long)]
        success_url: String,
    },
}

pub async fn run(cmd: BillingCmd, ctx: &CmdCtx) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    match cmd {
        BillingCmd::Info => {
            let resp = client.get("/billing").await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            let tier = data["billing_tier"].as_str().unwrap_or("-");
            let credit = data["credit_balance_usd"].as_f64().unwrap_or(0.0);
            let spend = data["monthly_spend_usd"].as_f64().unwrap_or(0.0);
            let status = data["subscription_status"].as_str().unwrap_or("none");

            println!("Billing Info");
            println!("{:-<40}", "");
            println!("{:<22} {:>12}", "Tier", tier);
            println!("{:<22} {:>12}", "Credit balance", format!("${:.2}", credit));
            println!("{:<22} {:>12}", "Monthly spend", format!("${:.6}", spend));
            if status != "none" {
                println!("{:<22} {:>12}", "Subscription", status);
            }
        }

        BillingCmd::Checkout { plan, success_url } => {
            #[derive(serde::Serialize)]
            struct Req {
                plan: String,
                success_url: String,
            }
            let resp = client.post("/billing/checkout", &Req { plan, success_url }).await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            if let Some(url) = data["checkout_url"].as_str() {
                println!("{} Checkout created.", "OK".green().bold());
                println!("  Open: {}", url.bold());
            }
        }

        BillingCmd::Cancel => {
            if !ctx.confirm("Cancel your subscription? You'll keep access until the end of the billing period.") {
                println!("Cancelled.");
                return Ok(());
            }
            client.delete("/billing/subscription").await?;
            if ctx.is_json() {
                println!(r#"{{"ok": true}}"#);
            } else {
                println!("{} Subscription cancelled.", "OK".green().bold());
            }
        }

        BillingCmd::Topup { amount, success_url } => {
            #[derive(serde::Serialize)]
            struct Req {
                amount_usd: f64,
                success_url: String,
            }
            let resp = client.post("/billing/topup", &Req { amount_usd: amount, success_url }).await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            if let Some(url) = data["checkout_url"].as_str() {
                println!("{} Top-up checkout created (${:.2}).", "OK".green().bold(), amount);
                println!("  Open: {}", url.bold());
            }
        }
    }

    Ok(())
}
