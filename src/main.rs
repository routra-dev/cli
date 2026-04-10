use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

mod auth;
mod client;
mod commands;
mod config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable table output
    Table,
    /// Machine-readable JSON output
    Json,
}

#[derive(Parser)]
#[command(
    name = "routra",
    about = "Routra CLI - One API. Every GPU Cloud.",
    version,
    propagate_version = true
)]
struct Cli {
    /// Routra API key (overrides ROUTRA_API_KEY env and config file)
    #[arg(long, env = "ROUTRA_API_KEY", global = true, hide_env_values = true)]
    api_key: Option<String>,

    /// API base URL (default: https://api.routra.dev/v1)
    #[arg(long, env = "ROUTRA_BASE_URL", global = true)]
    base_url: Option<String>,

    /// Output format
    #[arg(long, global = true, default_value = "table", value_enum)]
    output: OutputFormat,

    /// Skip confirmation prompts (for CI/scripting)
    #[arg(long, short = 'y', global = true)]
    yes: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with Routra (stores token in ~/.routra/config.json)
    Login,
    /// Log out and remove stored credentials
    Logout,
    /// Show current config and verify API key
    Whoami,

    /// Manage API keys
    Keys {
        #[command(subcommand)]
        action: commands::keys::KeysCmd,
    },

    /// Manage routing policies
    Policy {
        #[command(subcommand)]
        action: commands::policy::PolicyCmd,
    },

    /// View usage statistics for current billing period
    Usage,

    /// View cost breakdown for current billing period
    Cost,

    /// Manage batch inference jobs
    Batch {
        #[command(subcommand)]
        action: commands::batch::BatchCmd,
    },

    /// View billing info, checkout, and credit topup
    Billing {
        #[command(subcommand)]
        action: commands::billing::BillingCmd,
    },

    /// Manage webhook endpoints
    Webhooks {
        #[command(subcommand)]
        action: commands::webhooks::WebhooksCmd,
    },

    /// Manage Bring-Your-Own-Key provider keys
    #[command(name = "provider-keys")]
    ProviderKeys {
        #[command(subcommand)]
        action: commands::provider_keys::ProviderKeysCmd,
    },

    /// View recent request log
    Requests {
        /// Number of results to return
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Offset for pagination
        #[arg(long, default_value = "0")]
        offset: u32,
    },

    /// View audit log
    #[command(name = "audit-log")]
    AuditLog {
        /// Number of results to return
        #[arg(long, default_value = "50")]
        limit: u32,
        /// Offset for pagination
        #[arg(long, default_value = "0")]
        offset: u32,
    },

    /// List available providers and their status
    Providers,

    /// Browse the model catalog
    Models,

    /// Manage notification preferences and inbox
    Notifications {
        #[command(subcommand)]
        action: commands::notifications::NotificationsCmd,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let ctx = commands::CmdCtx {
        api_key: cli.api_key,
        base_url: cli.base_url,
        output: cli.output,
        yes: cli.yes,
    };

    match cli.command {
        Commands::Login => auth::login().await,
        Commands::Logout => auth::logout(),
        Commands::Whoami => commands::whoami::run(&ctx).await,
        Commands::Keys { action } => commands::keys::run(action, &ctx).await,
        Commands::Policy { action } => commands::policy::run(action, &ctx).await,
        Commands::Usage => commands::usage::run(&ctx).await,
        Commands::Cost => commands::cost::run(&ctx).await,
        Commands::Batch { action } => commands::batch::run(action, &ctx).await,
        Commands::Billing { action } => commands::billing::run(action, &ctx).await,
        Commands::Webhooks { action } => commands::webhooks::run(action, &ctx).await,
        Commands::ProviderKeys { action } => commands::provider_keys::run(action, &ctx).await,
        Commands::Requests { limit, offset } => commands::requests::run(&ctx, limit, offset).await,
        Commands::AuditLog { limit, offset } => commands::audit_log::run(&ctx, limit, offset).await,
        Commands::Providers => commands::providers::run(&ctx).await,
        Commands::Models => commands::models::run(&ctx).await,
        Commands::Notifications { action } => commands::notifications::run(&ctx, action).await,
    }
}
