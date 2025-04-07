use anyhow::Result;
use crate::llm::{LlmClient, LlmConfig, LlmProvider};
use tracing::info;

/// Configuración de la aplicación
pub struct AppConfig {
    /// Modelo LLM a utilizar
    pub model: String,
    /// URL de la API de LLM
    pub api_url: String,
    /// Nivel de verbosidad
    pub verbose: u8,
    /// Cliente LLM configurado
    pub llm_client: Option<LlmClient>,
}

impl AppConfig {
    /// Crear una nueva configuración desde los parámetros de la CLI
    pub fn new(model: String, api_url: String, verbose: u8, no_llm: bool) -> Result<Self> {
        // Configurar nivel de verbosidad
        match verbose {
            0 => println!("Modo normal"),
            1 => println!("Modo verbose"),
            _ => println!("Modo debug"),
        }
        
        // Configurar cliente LLM si no está desactivado
        let llm_client = if !no_llm {
            match LlmClient::new(LlmConfig {
                provider: LlmProvider::Ollama,
                api_url: api_url.clone(),
                model: model.clone(),
                temperature: 0.7,
                timeout_seconds: 30,
                max_tokens: Some(1024),
            }) {
                Ok(client) => {
                    info!("Cliente LLM inicializado con modelo {}", model);
                    Some(client)
                },
                Err(e) => {
                    println!("⚠️ Error al inicializar el cliente LLM: {}. Continuando sin análisis LLM.", e);
                    None
                }
            }
        } else {
            info!("Integración con LLM desactivada");
            None
        };
        
        Ok(Self {
            model,
            api_url,
            verbose,
            llm_client,
        })
    }
} 
