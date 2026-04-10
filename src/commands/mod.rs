pub mod audit_log;
pub mod batch;
pub mod billing;
pub mod cost;
pub mod keys;
pub mod models;
pub mod notifications;
pub mod policy;
pub mod provider_keys;
pub mod providers;
pub mod requests;
pub mod usage;
pub mod webhooks;
pub mod whoami;

use crate::OutputFormat;

/// Shared context passed to all commands.
pub struct CmdCtx {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub output: OutputFormat,
    pub yes: bool,
}

impl CmdCtx {
    pub fn is_json(&self) -> bool {
        self.output == OutputFormat::Json
    }

    /// Prompt for confirmation. Returns true if user confirms or `--yes` is set.
    pub fn confirm(&self, msg: &str) -> bool {
        if self.yes {
            return true;
        }
        print!("{} [y/N] ", msg);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        let mut answer = String::new();
        std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut answer).ok();
        answer.trim().eq_ignore_ascii_case("y")
    }
}
