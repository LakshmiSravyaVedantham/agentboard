use agentboard::config::Config;
use agentboard::auth::generate_pairing_code;
use agentboard::agent::registry::BackendRegistry;
use agentboard::orchestrator::Orchestrator;
use agentboard::state::AppState;
use agentboard::server::build_router;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Semaphore};
use chrono::Utc;
use uuid::Uuid;

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
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { config: config_path } => {
            let config = if std::path::Path::new(&config_path).exists() {
                Config::load(std::path::Path::new(&config_path)).unwrap_or_default()
            } else {
                Config::default()
            };

            let registry = BackendRegistry::from_config(&config);
            let orchestrator = Orchestrator::new(&config);
            let pairing_code = generate_pairing_code();
            let jwt_secret = Uuid::new_v4().to_string();
            let (ws_tx, _) = broadcast::channel(1024);
            let max_teams = config.limits.max_concurrent_teams;

            let state = Arc::new(AppState {
                config: config.clone(),
                registry,
                orchestrator,
                teams: Arc::new(RwLock::new(HashMap::new())),
                processes: Arc::new(RwLock::new(HashMap::new())),
                pending_plans: Arc::new(RwLock::new(HashMap::new())),
                concurrency_semaphore: Arc::new(Semaphore::new(max_teams)),
                pairing_code: pairing_code.clone(),
                jwt_secret,
                ws_broadcast: ws_tx,
                start_time: Utc::now(),
            });

            let app = build_router(state);
            let addr = format!("{}:{}", config.server.host, config.server.port);

            println!("┌────────────────────────────────────┐");
            println!("│  Agentboard v{}               │", env!("CARGO_PKG_VERSION"));
            println!("│  Running at http://{}     │", addr);
            println!("│                                    │");
            println!("│  Pairing code: {}             │", pairing_code);
            println!("│  Enter this on your phone to pair  │");
            println!("└────────────────────────────────────┘");

            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        }
    }
}
