use std::path::PathBuf;
use clap::{Parser, Subcommand};

/// Argumentos de línea de comandos para ShadowTrace
#[derive(Parser, Debug)]
#[clap(author, version, about = "Una aplicación TUI con características ASCII avanzadas")]
pub struct Cli {
    /// Archivos a procesar
    #[clap(name = "FILES")]
    pub files: Vec<PathBuf>,

    /// Modo verboso
    #[clap(short, long)]
    pub verbose: bool,

    /// Tema de color a utilizar
    #[clap(short, long, value_name = "TEMA", default_value = "default")]
    pub theme: String,

    /// Intervalo de actualización en milisegundos
    #[clap(short, long, value_name = "MS", default_value = "250")]
    pub interval: u64,

    /// Subcomandos
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

/// Subcomandos disponibles
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Iniciar en modo interactivo completo
    Interactive {
        /// Archivo de configuración
        #[clap(short, long, value_name = "ARCHIVO")]
        config: Option<PathBuf>,
    },
    /// Mostrar estadísticas rápidas
    Stats {
        /// Tipo de estadísticas
        #[clap(short, long, value_name = "TIPO", default_value = "basic")]
        type_: String,
        
        /// Formato de salida
        #[clap(short, long, value_name = "FORMATO", default_value = "text")]
        format: String,
    },
    /// Visualizar datos con gráficos ASCII
    Visualize {
        /// Tipo de visualización
        #[clap(short, long, value_name = "TIPO", default_value = "chart")]
        type_: String,
        
        /// Ancho de la visualización
        #[clap(short, long, value_name = "COLUMNAS", default_value = "80")]
        width: u16,
        
        /// Alto de la visualización
        #[clap(short, long, value_name = "FILAS", default_value = "24")]
        height: u16,
    },
}

/// Analizar argumentos de línea de comandos
pub fn parse_args() -> Cli {
    Cli::parse()
}

/// Configuración de la aplicación
#[derive(Debug, Clone)]
pub struct Config {
    /// Ruta del archivo de configuración
    pub config_path: Option<PathBuf>,
    /// Intervalo de actualización
    pub update_interval: u64,
    /// Modo verboso
    pub verbose: bool,
    /// Tema de color
    pub theme: String,
    /// Archivos a procesar
    pub files: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: None,
            update_interval: 250,
            verbose: false,
            theme: "default".to_string(),
            files: Vec::new(),
        }
    }
}

impl Config {
    /// Crear una nueva configuración a partir de los argumentos CLI
    pub fn from_cli(cli: &Cli) -> Self {
        let mut config = Config::default();
        
        config.update_interval = cli.interval;
        config.verbose = cli.verbose;
        config.theme = cli.theme.clone();
        config.files = cli.files.clone();
        
        if let Some(Commands::Interactive { config: config_path }) = &cli.command {
            config.config_path = config_path.clone();
        }
        
        config
    }
    
    /// Cargar configuración desde un archivo
    pub fn load_from_file(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
        // Esta función cargaría la configuración desde un archivo externo
        // Por ahora simplemente actualizamos la ruta
        self.config_path = Some(path.clone());
        Ok(())
    }
} 
