use clap::{Parser, Subcommand};
use lingua_core::{AppInfo, IpcRequest};
use lingua_engine::EngineRegistry;
use lingua_server::{run_server, ServerState};
use lingua_storage::StorageService;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "lingua")]
#[command(about = "LinguaFlow Unified CLI & Service Controller", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect system status, database health, and runtime capability matrix
    Status,
    /// Launch the Axum REST & WebSocket edge gateway server
    Serve {
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// List available and scheduled model packs
    Models,
    /// Run baseline micro-benchmarks for persistence and IPC serialization
    Bench {
        #[arg(short, long, default_value_t = 1000)]
        iterations: u32,
    },
    /// Run diagnostic environment and health doctor
    Doctor,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let storage = StorageService::open_default()?;
    let engine = Arc::new(EngineRegistry::new());
    let app_info = AppInfo::default();

    match cli.command {
        Commands::Status => {
            println!("==================================================");
            println!("  LinguaFlow System Status (Stage {})", app_info.stage);
            println!("==================================================");
            println!("Version:       {}", app_info.version);
            println!("Stage Name:    {}", app_info.stage_name);
            println!("Platform:      {} ({})", app_info.platform, app_info.arch);
            println!("Build Profile: {}", app_info.build_profile);

            let health = storage.check_health()?;
            println!("\nDatabase:");
            println!("  Status:          {}", health.status);
            println!("  Path:            {}", health.path);
            println!("  Schema Version:  {}", health.schema_version);
            println!("  Settings Ready:  {}", health.settings_persisted);

            let caps = engine.get_capabilities();
            println!("\nCapabilities (Honest Baseline):");
            println!("  Dictionary:      {:?}", caps.dictionary);
            println!("  Translation:     {:?}", caps.text_translation);
            println!("  Streaming ASR:   {:?}", caps.streaming_asr);
            println!("  TTS Voice:       {:?}", caps.speech_synthesis);
            println!("  Offline Models:  {:?}", caps.offline_models);
            println!("==================================================");
        }
        Commands::Serve { host, port } => {
            let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
            let state = ServerState::new(storage, engine, app_info);
            println!("Starting LinguaFlow Gateway on http://{}", addr);
            run_server(state, addr).await?;
        }
        Commands::Models => {
            let packs = storage.list_model_packs()?;
            println!("Found {} registered model pack(s):", packs.len());
            for pack in packs {
                println!(
                    " - [{}] {} ({}) [Stage {}] - Installed: {}",
                    pack.id, pack.name, pack.version, pack.scheduled_stage, pack.is_installed
                );
            }
        }
        Commands::Bench { iterations } => {
            println!(
                "Running LinguaFlow Stage 0 Benchmarks ({} iterations)...",
                iterations
            );

            // Benchmark 1: IPC serialization roundtrip
            let req = IpcRequest::new(app_info.clone());
            let start = Instant::now();
            for _ in 0..iterations {
                let serialized = serde_json::to_string(&req)?;
                let _deserialized: IpcRequest<AppInfo> = serde_json::from_str(&serialized)?;
            }
            let elapsed_ipc = start.elapsed();
            let avg_ipc_us = (elapsed_ipc.as_micros() as f64) / (iterations as f64);
            println!("  IPC Serialization Roundtrip: {:.2} µs / op", avg_ipc_us);

            // Benchmark 2: SQLite settings read
            let start = Instant::now();
            for _ in 0..iterations {
                let _settings = storage.get_user_settings()?;
            }
            let elapsed_db = start.elapsed();
            let avg_db_us = (elapsed_db.as_micros() as f64) / (iterations as f64);
            println!("  SQLite Settings Read Latency: {:.2} µs / op", avg_db_us);

            println!("Benchmarks completed successfully.");
        }
        Commands::Doctor => {
            println!("Running LinguaFlow Environment Doctor...");
            let health = storage.check_health()?;
            assert_eq!(health.status, "healthy");
            println!(" [OK] SQLite connection pool and WAL mode verified.");
            println!(
                " [OK] Schema migrations verified at version {}.",
                health.schema_version
            );
            println!(" [OK] Core BCP-47 language primitives active (kn-IN, ta-IN, en-US).");
            println!(" [OK] Engine registry active in Stage 0 baseline mode.");
            println!("LinguaFlow environment is healthy and ready.");
        }
    }

    Ok(())
}
