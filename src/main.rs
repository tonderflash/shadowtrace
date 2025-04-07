use std::path::PathBuf;
use std::error::Error;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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

// Manejar eventos de teclado
fn handle_key_events(key_event: KeyEvent, app: &mut App) -> bool {
    match app.current_state {
        AppState::Dashboard => handle_dashboard_keys(key_event, app),
        AppState::ProcessMonitor => handle_process_monitor_keys(key_event, app),
        AppState::FileMonitor => handle_file_monitor_keys(key_event, app),
        AppState::NetworkMonitor => handle_network_monitor_keys(key_event, app),
        AppState::Reports => handle_reports_keys(key_event, app),
        AppState::Help => handle_help_keys(key_event, app),
    }
}

// Manejar eventos de teclado en la pantalla de monitor de procesos
fn handle_process_monitor_keys(key_event: KeyEvent, app: &mut App) -> bool {
    match key_event.code {
        // Teclas de navegación general
        KeyCode::Esc => {
            app.current_state = AppState::Dashboard;
            app.selected_pid = None;
            app.status_message = None;
            app.cpu_history.clear();
            app.memory_history.clear();
            if app.is_monitoring_active {
                app.stop_monitoring();
            }
            true
        },
        KeyCode::Tab => {
            // Alternar entre pestañas
            if app.process_monitor_tab == 0 {
                app.process_monitor_tab = 1;
            } else {
                app.process_monitor_tab = 0;
            }
            true
        },
        // Navegación en la lista de procesos
        KeyCode::Up => {
            // Si estamos en la pestaña de análisis LLM y hay análisis, desplazar el texto
            if app.process_monitor_tab == 1 && app.process_llm_analysis.is_some() {
                app.handle_llm_text_scroll(KeyCode::Up);
            } else {
                // Si no, navegar por la lista de procesos
                let processes = &app.processes;
                if !processes.is_empty() {
                    let i = match app.list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                processes.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    app.list_state.select(Some(i));
                    
                    // Actualizar selección
                    let selected_process = &processes[i];
                    app.selected_pid = Some(selected_process.pid);
                    app.status_message = Some(format!("Proceso seleccionado: {} (PID: {})", 
                        selected_process.name, selected_process.pid));
                }
            }
            true
        },
        KeyCode::Down => {
            // Si estamos en la pestaña de análisis LLM y hay análisis, desplazar el texto
            if app.process_monitor_tab == 1 && app.process_llm_analysis.is_some() {
                app.handle_llm_text_scroll(KeyCode::Down);
            } else {
                // Si no, navegar por la lista de procesos
                let processes = &app.processes;
                if !processes.is_empty() {
                    let i = match app.list_state.selected() {
                        Some(i) => {
                            if i >= processes.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    app.list_state.select(Some(i));
                    
                    // Actualizar selección
                    let selected_process = &processes[i];
                    app.selected_pid = Some(selected_process.pid);
                    app.status_message = Some(format!("Proceso seleccionado: {} (PID: {})", 
                        selected_process.name, selected_process.pid));
                }
            }
            true
        },
        // Teclas adicionales para navegación de scroll en análisis LLM
        KeyCode::PageUp => {
            if app.process_monitor_tab == 1 && app.process_llm_analysis.is_some() {
                app.handle_llm_text_scroll(KeyCode::PageUp);
                true
            } else {
                false
            }
        },
        KeyCode::PageDown => {
            if app.process_monitor_tab == 1 && app.process_llm_analysis.is_some() {
                app.handle_llm_text_scroll(KeyCode::PageDown);
                true
            } else {
                false
            }
        },
        KeyCode::Home => {
            if app.process_monitor_tab == 1 && app.process_llm_analysis.is_some() {
                app.handle_llm_text_scroll(KeyCode::Home);
                true
            } else {
                false
            }
        },
        KeyCode::End => {
            if app.process_monitor_tab == 1 && app.process_llm_analysis.is_some() {
                app.handle_llm_text_scroll(KeyCode::End);
                true
            } else {
                false
            }
        },
        // Otras teclas...
        // ... resto del código existente ...
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Inicializar logger con más detalles y filtro basado en nivel de verbose
    let filter = match std::env::var("RUST_LOG") {
        Ok(val) => EnvFilter::new(val),
        Err(_) => {
            // Si RUST_LOG no está definido, usar valores basados en verbose
            EnvFilter::new(format!("shadowtrace={}", match Cli::parse().verbose {
                0 => "info",
                1 => "debug",
                _ => "trace",
            }))
        }
    };
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();
    
    // Log inicial para verificar que está funcionando
    tracing::info!("ShadowTrace iniciando...");
    tracing::debug!("Nivel de depuración activado");
    
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
