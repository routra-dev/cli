use anyhow::Result;
use clap::{Parser, Subcommand};

mod auth;
mod client;
mod commands;
mod config;

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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with Routra (stores token in ~/.routra/config.json)
    Login,
    /// Log out and remove stored credentials
    Logout,

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

    /// View usage statistics
    Usage {
        /// Filter by model name
        #[arg(long)]
        model: Option<String>,
        /// Number of days to look back (default: 30)
        #[arg(long, default_value = "30")]
        days: u32,
    },

    /// View cost breakdown
    Cost {
        /// Break down by: provider | model | key
        #[arg(long, default_value = "model")]
        breakdown: String,
        /// Number of days to look back (default: 30)
        #[arg(long, default_value = "30")]
        days: u32,
    },

    /// Manage batch inference jobs
    Batch {
        #[command(subcommand)]
        action: commands::batch::BatchCmd,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login => auth::login().await,
        Commands::Logout => auth::logout(),
        Commands::Keys { action } => commands::keys::run(action, &cli.api_key, &cli.base_url).await,
        Commands::Policy { action } => commands::policy::run(action, &cli.api_key, &cli.base_url).await,
        Commands::Usage { model, days } => commands::usage::run(model, days, &cli.api_key, &cli.base_url).await,
        Commands::Cost { breakdown, days } => commands::cost::run(breakdown, days, &cli.api_key, &cli.base_url).await,
        Commands::Batch { action } => commands::batch::run(action, &cli.api_key, &cli.base_url).await,
    }
}
