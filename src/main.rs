use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agentboard", version, about = "Orchestrate AI coding agents from your phone")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the agentboard server
    Serve {
        #[arg(short, long, default_value = "agentboard.toml")]
        config: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { config } => {
            println!("Loading config from: {}", config);
            println!("Agentboard starting...");
        }
    }
}
