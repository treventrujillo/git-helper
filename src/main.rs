use clap::{Parser, Subcommand};

mod commands;
mod config;
mod gitx;
mod plan;
mod util;

#[derive(Parser)]
struct Cli {
    #[arg(global = true, long, default_value_t = true)]
    dry_run: bool,

    #[arg(global = true, long)]
    config: Option<String>,

    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Sync {
        #[arg(long)]
        main: Option<String>,

        #[arg(long)]
        push: bool,

        #[arg(long)]
        non_interactive: bool,
    },
}

fn main() -> anyhow::Result<()> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Sync {
            main,
            push,
            non_interactive,
        } => commands::sync::run_sync(commands::sync::SyncArgs {
            dry_run: cli.dry_run,
            main_override: main,
            push,
            non_interactive,
            config_path: cli.config,
        }),
    }
}
