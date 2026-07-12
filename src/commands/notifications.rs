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

            // Server contract: InboxItemResponse { id, event_type, title, body, is_read, created_at }
            println!("{:<36}  {:<6}  {:<20}  MESSAGE", "ID", "READ", "CREATED");
            for n in items {
                let id = n["id"].as_str().unwrap_or("-");
                let read = if n["is_read"].as_bool().unwrap_or(false) {
                    "yes"
                } else {
                    "no"
                };
                let created = n["created_at"].as_str().unwrap_or("-");
                let title = n["title"].as_str().unwrap_or("-");
                let body = n["body"].as_str().unwrap_or("");
                println!(
                    "{:<36}  {:<6}  {:<20}  {}{}{}",
                    id,
                    read,
                    &created[..19.min(created.len())],
                    title,
                    if body.is_empty() { "" } else { " — " },
                    body,
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

            let count = data["unread_count"].as_u64().unwrap_or(0);
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

            // Server contract: NotificationPreferenceResponse { event_type, email_enabled, in_app_enabled }
            println!("{:<24}  {:<8}  EMAIL", "EVENT TYPE", "IN-APP");
            for p in items {
                let event = p["event_type"].as_str().unwrap_or("-");
                let in_app = if p["in_app_enabled"].as_bool().unwrap_or(false) {
                    "on"
                } else {
                    "off"
                };
                let email = if p["email_enabled"].as_bool().unwrap_or(false) {
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
            // Server contract: UpdatePreferenceRequest requires BOTH
            // email_enabled and in_app_enabled. When the user sets only one
            // flag, fetch the current preference and keep the other value
            // (defaulting to enabled if no preference exists yet).
            let (cur_in_app, cur_email) = if in_app.is_none() || email.is_none() {
                let resp = client.get("/notifications/preferences").await?;
                let prefs: serde_json::Value = resp.json().await?;
                let existing = prefs
                    .as_array()
                    .and_then(|arr| {
                        arr.iter()
                            .find(|p| p["event_type"].as_str() == Some(event_type.as_str()))
                    })
                    .cloned()
                    .unwrap_or_default();
                (
                    existing["in_app_enabled"].as_bool().unwrap_or(true),
                    existing["email_enabled"].as_bool().unwrap_or(true),
                )
            } else {
                (true, true) // unused - both flags provided
            };

            let body = serde_json::json!({
                "event_type": event_type,
                "in_app_enabled": in_app.unwrap_or(cur_in_app),
                "email_enabled": email.unwrap_or(cur_email),
            });

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
