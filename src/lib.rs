// Módulos principales
pub mod ui;
pub mod process;
pub mod file_monitor;
pub mod network;
pub mod reports;
pub mod app;
pub mod cli;
pub mod config;
pub mod commands;
pub mod error;
pub mod llm;

// Reexportaciones útiles para los usuarios de la biblioteca
pub use app::App;
pub use cli::Config;

/// Versión de la aplicación
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Nombre de la aplicación
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

/// Descripción de la aplicación
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Autor de la aplicación
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS"); 
