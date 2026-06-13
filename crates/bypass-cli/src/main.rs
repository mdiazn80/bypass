mod cmd;
mod tui;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "bypass",
    version,
    about = "Manage and inject Bypass credential contexts"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Set the global active credential context.
    Use {
        /// Name of the context to activate.
        context: String,
    },
    /// List available contexts and show which one is active.
    List,
    /// Run a command with the active context's variables injected.
    Run {
        /// The command and its arguments, after `--`.
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
        command: Vec<String>,
    },
    /// Launch a subshell with the active context's variables injected.
    Shell,
    /// Export the whole vault to a passphrase-encrypted file for migration.
    Export {
        /// Destination file.
        file: PathBuf,
    },
    /// Import contexts from a passphrase-encrypted export file.
    Import {
        /// Source file.
        file: PathBuf,
    },
    /// Launch the interactive TUI (also the default when run without arguments).
    Tui,
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Command::Use { context }) => cmd::use_context(&context),
        Some(Command::List) => cmd::list(),
        Some(Command::Run { command }) => cmd::run(&command),
        Some(Command::Shell) => cmd::shell(),
        Some(Command::Export { file }) => cmd::export(&file),
        Some(Command::Import { file }) => cmd::import(&file),
        Some(Command::Tui) | None => tui::run(),
    };

    if let Err(e) = result {
        eprintln!("bypass: {e:#}");
        std::process::exit(1);
    }
}
