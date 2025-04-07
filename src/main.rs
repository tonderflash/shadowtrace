use std::path::PathBuf;
use std::error::Error;
use clap::{Parser, Subcommand};

use crate::config::AppConfig;
use crate::commands::{monitor_process, audit_binary, monitor_system};
use crate::ui::{App, Tui};

mod ui;
mod app;
mod process;
mod file_monitor;
mod network;
mod reports;
mod config;
mod commands;
mod error;
mod llm;

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
    command: Option<Commands>,

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

    /// Iniciar en modo TUI
    #[arg(long)]
    tui: bool,
}

// Comandos CLI disponibles
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
        
        /// Duración del monitoreo en segundos (0 = indefinido)
        #[arg(short, long, default_value = "0")]
        duration: u64,
        
        /// Intervalo de muestreo en segundos
        #[arg(short, long, default_value = "1")]
        interval: u64,
    },
    
    /// Auditar un binario
    Audit {
        /// Ruta al binario a auditar
        #[arg(required = true)]
        binary: PathBuf,
        
        /// Argumentos para el binario
        #[arg(short, long)]
        args: Option<Vec<String>>,
        
        /// Tiempo máximo de ejecución en segundos
        #[arg(short, long, default_value = "60")]
        timeout: u64,
    },
    
    /// Monitorear actividad del sistema
    System {
        /// Monitorear en tiempo real
        #[arg(short, long)]
        watch: bool,
        
        /// Duración del monitoreo en segundos
        #[arg(short, long, default_value = "60")]
        duration: u64,
        
        /// Solo mostrar actividad sospechosa
        #[arg(short, long)]
        suspicious_only: bool,
    },
}

/// Función para ejecutar la interfaz de usuario de terminal (TUI)
fn run_tui_mode(config: &AppConfig) -> Result<(), Box<dyn Error>> {
    // Crear una instancia de la aplicación TUI
    let mut app = App::new();
    
    // Configurar app con AppConfig
    if let Some(client) = &config.llm_client {
        app.status_message = Some("Cliente LLM conectado".to_string());
    }
    
    // Crear e inicializar la terminal TUI
    let mut tui = Tui::new()?;
    tui.init()?;
    
    // Ejecutar el loop principal de la UI
    let result = tui.run(&mut app);
    
    // Restaurar terminal
    if let Err(e) = tui.exit() {
        eprintln!("Error al restaurar terminal: {}", e);
    }
    
    // Propagar el resultado del loop principal
    result.map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn Error>)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Inicializar logger
    tracing_subscriber::fmt::init();
    
    // Parsear argumentos CLI
    let cli = Cli::parse();
    
    // Crear configuración global
    let config = AppConfig::new(
        cli.model.clone(), 
        cli.api_url.clone(), 
        cli.verbose, 
        cli.no_llm
    )?;
    
    // Determinar si se debe ejecutar en modo TUI
    let use_tui = cli.tui || cli.command.is_none();
    
    if use_tui {
        // Ejecutar en modo TUI
        run_tui_mode(&config)?;
        return Ok(());
    }
    
    // Modo CLI normal
    match cli.command {
        Some(Commands::Monitor { pid, name, duration, interval }) => {
            // Ejecutar monitoreo
            monitor_process(&pid, &name, duration, interval, &config).await?;
        },
        Some(Commands::Audit { binary, args, timeout }) => {
            // Ejecutar auditoría
            audit_binary(&binary, &args, timeout, &config).await?;
        },
        Some(Commands::System { watch, duration, suspicious_only }) => {
            // Ejecutar monitoreo de sistema
            monitor_system(watch, duration, suspicious_only, &config).await?;
        },
        None => {
            // No debería llegar aquí si use_tui es true cuando command es None
            println!("Modo TUI no implementado todavía");
        }
    }
    
    Ok(())
}
