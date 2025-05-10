pub mod ai;
pub mod commands;
pub mod notes;
pub mod tui;
pub mod web;

use clap::{Parser, Subcommand};
use std::fs;
use std::io;

pub type MyError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Parser, Debug)]
#[command(
    name = "nrs",
    version = "0.4.3",
    about = "Rust-based TUI & Web for Notes"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new note
    New { title: String },
    /// Run TUI
    Tui,
    /// Start the web server
    Serve {
        #[arg(short, long, default_value_t = 4321)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // Ensure the ~/notes directory exists
    let ndir = notes::notes_dir();
    if !ndir.exists() {
        fs::create_dir(&ndir).map_err(|e| {
            eprintln!("Cannot create ~/notes: {}", e);
            e
        })?;
    }

    match cli.command {
        Commands::New { title } => {
            if let Err(e) = notes::create_new_note(&title) {
                eprintln!("Error creating note: {}", e);
            }
        }
        Commands::Tui => {
            if let Err(e) = tui::run_tui() {
                eprintln!("Error in TUI: {}", e);
            }
        }
        Commands::Serve { port } => {
            web::serve_notes(port).await?;
        }
    }

    Ok(())
}
