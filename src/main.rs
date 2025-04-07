use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod process;
mod file_monitor;
mod network;
mod llm;
mod reports;
mod config;
mod commands;
mod error;

use config::AppConfig;
use commands::{monitor_process, audit_binary, monitor_system};

// CLI principal
#[derive(Parser)]
#[command(
    author, 
    version, 
    about = "ShadowTrace - Local AI-Powered Blackbox Debugger", 
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Nivel de verbosidad
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Modelo LLM a utilizar (llama2, mistral, orca, etc)
    #[arg(short, long, default_value = "llama2")]
    model: String,

    /// URL de la API de Ollama
    #[arg(long, default_value = "http://localhost:11434/api")]
    api_url: String,

    /// Desactivar integración LLM
    #[arg(long)]
    no_llm: bool,
}

// Subcomandos
#[derive(Subcommand)]
enum Commands {
    /// Monitorear un proceso específico
    Monitor {
        /// ID del proceso a monitorear
        #[arg(short, long)]
        pid: Option<u32>,

        /// Nombre del proceso a monitorear
        #[arg(short, long)]
        name: Option<String>,

        /// Duración del monitoreo en segundos (0 para infinito)
        #[arg(short, long, default_value = "60")]
        duration: u64,

        /// Intervalo de monitoreo en segundos
        #[arg(short, long, default_value = "1")]
        interval: u64,
    },

    /// Analizar un binario
    Audit {
        /// Ruta al binario a analizar
        #[arg(short, long)]
        binary: PathBuf,

        /// Argumentos a pasar al binario
        #[arg(short, long)]
        args: Option<Vec<String>>,

        /// Tiempo máximo de ejecución en segundos
        #[arg(short, long, default_value = "60")]
        timeout: u64,
    },

    /// Monitorear actividad del sistema
    System {
        /// Modo de monitoreo continuo
        #[arg(short, long)]
        watch: bool,

        /// Duración del monitoreo en segundos (0 para infinito)
        #[arg(short, long, default_value = "60")]
        duration: u64,

        /// Mostrar solo procesos sospechosos
        #[arg(short, long)]
        suspicious_only: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    tracing_subscriber::fmt::init();

    // Parsear argumentos de línea de comandos
    let cli = Cli::parse();
    
    // Crear configuración de la aplicación
    let config = AppConfig::new(cli.model, cli.api_url, cli.verbose, cli.no_llm)?;

    // Ejecutar el comando apropiado
    match &cli.command {
        Commands::Monitor { pid, name, duration, interval } => {
            monitor_process(pid, name, *duration, *interval, &config).await?;
        }
        Commands::Audit { binary, args, timeout } => {
            audit_binary(binary, args, *timeout, &config).await?;
        }
        Commands::System { watch, duration, suspicious_only } => {
            monitor_system(*watch, *duration, *suspicious_only, &config).await?;
        }
    }

    Ok(())
}
