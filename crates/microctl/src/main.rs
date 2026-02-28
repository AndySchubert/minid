//! `microctl` — CLI frontend for the minid container runtime.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

/// microctl — a minimal OCI container runtime CLI.
#[derive(Parser)]
#[command(name = "microctl", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new container from an OCI bundle.
    Create {
        /// Unique container identifier.
        id: String,
        /// Path to the OCI bundle directory.
        bundle: String,
    },
    /// Start a previously created container.
    Start {
        /// Container identifier.
        id: String,
    },
    /// Query the state of a container (JSON output).
    State {
        /// Container identifier.
        id: String,
    },
    /// Send a signal to a running container.
    Kill {
        /// Container identifier.
        id: String,
        /// Signal to send (e.g. SIGTERM, SIGKILL, 9). Default: SIGTERM.
        #[arg(short, long, default_value = "SIGTERM")]
        signal: String,
    },
    /// Delete a stopped container.
    Delete {
        /// Container identifier.
        id: String,
    },
}

fn main() -> Result<()> {
    // Initialise tracing (controlled via RUST_LOG env var).
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Create { id, bundle } => {
            let bundle_path = std::path::Path::new(&bundle)
                .canonicalize()
                .context("bundle path does not exist")?;

            let state = minid::Container::create(&id, &bundle_path)
                .context("failed to create container")?;

            let json = serde_json::to_string_pretty(&state)?;
            println!("{json}");
        }

        Commands::Start { id } => {
            minid::Container::start(&id).context("failed to start container")?;
            println!("container {id} started");
        }

        Commands::State { id } => {
            let state =
                minid::Container::state(&id).context("failed to query container state")?;
            let json = serde_json::to_string_pretty(&state)?;
            println!("{json}");
        }

        Commands::Kill { id, signal } => {
            let sig = parse_signal(&signal)?;
            minid::Container::kill(&id, sig).context("failed to kill container")?;
            println!("signal {signal} sent to container {id}");
        }

        Commands::Delete { id } => {
            minid::Container::delete(&id).context("failed to delete container")?;
            println!("container {id} deleted");
        }
    }

    Ok(())
}

/// Parse a signal name (e.g. "SIGTERM", "TERM") or number (e.g. "9") into a Signal.
fn parse_signal(s: &str) -> Result<minid::Signal> {
    use minid::Signal;

    // Try parsing as a number first.
    if let Ok(num) = s.parse::<i32>() {
        return Signal::try_from(num)
            .map_err(|_| anyhow::anyhow!("invalid signal number: {num}"));
    }

    // Strip optional "SIG" prefix and match.
    let name = s.strip_prefix("SIG").unwrap_or(s);
    match name.to_uppercase().as_str() {
        "TERM" => Ok(Signal::SIGTERM),
        "KILL" => Ok(Signal::SIGKILL),
        "INT" => Ok(Signal::SIGINT),
        "HUP" => Ok(Signal::SIGHUP),
        "USR1" => Ok(Signal::SIGUSR1),
        "USR2" => Ok(Signal::SIGUSR2),
        "STOP" => Ok(Signal::SIGSTOP),
        "CONT" => Ok(Signal::SIGCONT),
        _ => Err(anyhow::anyhow!("unknown signal: {s}")),
    }
}
