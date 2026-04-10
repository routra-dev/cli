use anyhow::Result;
use clap::Subcommand;

use crate::client::RoutraClient;
use super::CmdCtx;

#[derive(Subcommand)]
pub enum NotificationsCmd {
    /// List inbox notifications
    Inbox {
        /// Max items to return
        #[arg(long, default_value = "50")]
        limit: u32,
    },
    /// Get unread notification count
    UnreadCount,
    /// Mark a single notification as read
    MarkRead {
        /// Notification ID
        id: String,
    },
    /// Mark all notifications as read
    MarkAllRead,
    /// List notification preferences
    Preferences,
    /// Update a notification preference
    UpdatePreference {
        /// Event type (e.g. spend_alert, circuit_breaker)
        #[arg(long)]
        event_type: String,
        /// Enable in-app notifications
        #[arg(long)]
        in_app: Option<bool>,
        /// Enable email notifications
        #[arg(long)]
        email: Option<bool>,
    },
}

pub async fn run(ctx: &CmdCtx, cmd: NotificationsCmd) -> Result<()> {
    let client = RoutraClient::new(&ctx.api_key, &ctx.base_url)?;

    match cmd {
        NotificationsCmd::Inbox { limit } => {
            let resp = client
                .get(&format!("/notifications/inbox?limit={}", limit))
                .await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            let items = data.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
            if items.is_empty() {
                println!("Inbox is empty.");
                return Ok(());
            }

            println!("{:<36}  {:<6}  {:<20}  MESSAGE", "ID", "READ", "CREATED");
            for n in items {
                let id = n["id"].as_str().unwrap_or("-");
                let read = if n["read"].as_bool().unwrap_or(false) {
                    "yes"
                } else {
                    "no"
                };
                let created = n["created_at"].as_str().unwrap_or("-");
                let message = n["message"].as_str().unwrap_or("-");
                println!(
                    "{:<36}  {:<6}  {:<20}  {}",
                    id,
                    read,
                    &created[..19.min(created.len())],
                    message,
                );
            }

            Ok(())
        }
        NotificationsCmd::UnreadCount => {
            let resp = client.get("/notifications/inbox/unread-count").await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            let count = data["count"].as_u64().unwrap_or(0);
            println!("Unread notifications: {}", count);
            Ok(())
        }
        NotificationsCmd::MarkRead { id } => {
            client
                .post_empty(&format!("/notifications/inbox/{}/read", id))
                .await?;
            println!("Marked notification {} as read.", id);
            Ok(())
        }
        NotificationsCmd::MarkAllRead => {
            client.post_empty("/notifications/inbox/read-all").await?;
            println!("All notifications marked as read.");
            Ok(())
        }
        NotificationsCmd::Preferences => {
            let resp = client.get("/notifications/preferences").await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            let items = data.as_array().map(|a| a.as_slice()).unwrap_or(&[]);
            if items.is_empty() {
                println!("No notification preferences configured.");
                return Ok(());
            }

            println!("{:<24}  {:<8}  EMAIL", "EVENT TYPE", "IN-APP");
            for p in items {
                let event = p["event_type"].as_str().unwrap_or("-");
                let in_app = if p["in_app"].as_bool().unwrap_or(false) {
                    "on"
                } else {
                    "off"
                };
                let email = if p["email"].as_bool().unwrap_or(false) {
                    "on"
                } else {
                    "off"
                };
                println!("{:<24}  {:<8}  {}", event, in_app, email);
            }

            Ok(())
        }
        NotificationsCmd::UpdatePreference {
            event_type,
            in_app,
            email,
        } => {
            let mut body = serde_json::json!({ "event_type": event_type });
            if let Some(v) = in_app {
                body["in_app"] = serde_json::Value::Bool(v);
            }
            if let Some(v) = email {
                body["email"] = serde_json::Value::Bool(v);
            }

            let resp = client.put("/notifications/preferences", &body).await?;
            let data: serde_json::Value = resp.json().await?;

            if ctx.is_json() {
                println!("{}", serde_json::to_string_pretty(&data)?);
                return Ok(());
            }

            println!("Updated preference for '{}'.", event_type);
            Ok(())
        }
    }
}
