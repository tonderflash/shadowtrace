use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Proveedor de LLM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    /// Ollama (https://ollama.ai)
    Ollama,
    /// Directo a la API OpenAI compatible
    OpenAiCompatible,
}

/// Configuración para el cliente LLM
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Proveedor seleccionado
    pub provider: LlmProvider,
    /// URL base para la API
    pub api_url: String,
    /// Modelo a utilizar
    pub model: String,
    /// Temperatura (creatividad) del modelo
    pub temperature: f32,
    /// Timeout en segundos
    pub timeout_seconds: u64,
    /// Longitud máxima de salida
    pub max_tokens: Option<u32>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Ollama,
            api_url: "http://localhost:11434/api".to_string(),
            model: "llama2".to_string(),
            temperature: 0.5,
            timeout_seconds: 30,
            max_tokens: Some(512),
        }
    }
}

/// Solicitud a Ollama
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

/// Respuesta de Ollama u OpenAI
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    #[serde(default)]
    model: String,
    #[serde(default)]
    response: String,
    #[serde(default)]
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Option<Message>,
    #[serde(default)]
    index: i32,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

/// Cliente para interactuar con el LLM
pub struct LlmClient {
    config: LlmConfig,
    client: Client,
}

impl LlmClient {
    /// Crear un nuevo cliente LLM con la configuración especificada
    pub fn new(config: LlmConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .context("Error creando cliente HTTP")?;
        
        Ok(Self { config, client })
    }
    
    /// Analizar un proceso basado en información recopilada
    pub async fn analyze_process(&self, process_info: serde_json::Value) -> Result<String> {
        let prompt = format!(
            "Actúa como un analista de seguridad experto. Analiza la siguiente información \
            de un proceso en ejecución y determina su comportamiento, posibles intenciones, \
            y si hay actividad sospechosa. Sé específico y detallado. Si ves patrones \
            sospechosos, indícalos claramente.\n\nInformación del proceso:\n{}", 
            serde_json::to_string_pretty(&process_info)?
        );
        
        self.generate_response(&prompt).await
    }
    
    /// Analizar patrones de archivo de un proceso
    pub async fn analyze_file_activity(&self, file_events: serde_json::Value) -> Result<String> {
        let prompt = format!(
            "Actúa como un analista de seguridad. Revisa los siguientes eventos de acceso \
            a archivos de un proceso y determina patrones, intenciones, y posibles comportamientos \
            maliciosos. Señala cualquier actividad que parezca inusual o sospechosa.\n\n\
            Eventos de archivo:\n{}", 
            serde_json::to_string_pretty(&file_events)?
        );
        
        self.generate_response(&prompt).await
    }
    
    /// Analizar patrones de red de un proceso
    pub async fn analyze_network_activity(&self, network_events: serde_json::Value) -> Result<String> {
        let prompt = format!(
            "Actúa como un analista de seguridad de redes. Examina los siguientes eventos de red \
            de un proceso y determina patrones, posibles intenciones, y cualquier actividad sospechosa. \
            Si detectas indicadores de comportamiento malicioso, exfiltración de datos o comunicación \
            con servidores potencialmente maliciosos, indícalo claramente.\n\n\
            Eventos de red:\n{}", 
            serde_json::to_string_pretty(&network_events)?
        );
        
        self.generate_response(&prompt).await
    }
    
    /// Provee un análisis completo de un proceso
    pub async fn comprehensive_analysis(
        &self, 
        process_info: serde_json::Value,
        file_events: serde_json::Value,
        network_events: serde_json::Value
    ) -> Result<String> {
        // Combinar toda la información en un solo objeto JSON
        let mut combined = serde_json::Map::new();
        combined.insert("process".to_string(), process_info);
        combined.insert("file_activity".to_string(), file_events);
        combined.insert("network_activity".to_string(), network_events);
        
        let prompt = format!(
            "Eres un modelo de lenguaje. No necesitas capacidades de ejecución para hacer análisis útil.

A continuación te paso un reporte generado por un sistema llamado ShadowTrace. El reporte contiene texto estructurado con información de procesos, archivos accedidos y conexiones de red. No necesitas analizar archivos ni ejecutar código, solo interpretar lo que ya está escrito.

Tu tarea es:
	• Leer el reporte y explicar en lenguaje humano técnico qué está haciendo el proceso monitoreado.
	• Comentar si el comportamiento es típico o sospechoso.
	• Hacer hipótesis razonables sobre el propósito del proceso basándote en los datos observados (ej. si parece un navegador, si hay persistencia, si hay tráfico de red anómalo, etc.).
	• Puedes mencionar si algo parece malicioso, sospechoso o simplemente automatizado.
	• Tu análisis puede ser especulativo, pero bien fundamentado.
	• NO digas que no puedes hacer el análisis. NO digas que eres solo un modelo de lenguaje. Ya lo sabemos. Estás aquí para interpretar texto. Hazlo como si fueras un analista de seguridad con experiencia.

Aquí está el reporte:\n{}", 
            serde_json::to_string_pretty(&Value::Object(combined))?
        );
        
        self.generate_response(&prompt).await
    }
    
    /// Generar una respuesta basada en el prompt
    pub async fn generate_response(&self, prompt: &str) -> Result<String> {
        match self.config.provider {
            LlmProvider::Ollama => self.generate_ollama_response(prompt).await,
            LlmProvider::OpenAiCompatible => self.generate_openai_compatible_response(prompt).await,
        }
    }
    
    /// Generar una respuesta utilizando Ollama
    async fn generate_ollama_response(&self, prompt: &str) -> Result<String> {
        let request = if self.config.api_url.contains("/v1/chat/completions") {
            // Formato compatible con OpenAI
            let openai_request = serde_json::json!({
                "model": self.config.model.clone(),
                "messages": [
                    {
                        "role": "system",
                        "content": "Eres un asistente de seguridad informática con amplio conocimiento en análisis de comportamiento de procesos y detección de amenazas."
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ],
                "temperature": self.config.temperature,
                "max_tokens": self.config.max_tokens,
            });
            openai_request
        } else {
            // Formato de Ollama
            let ollama_request = OllamaRequest {
                model: self.config.model.clone(),
                prompt: prompt.to_string(),
                temperature: self.config.temperature,
                max_tokens: self.config.max_tokens,
            };
            serde_json::to_value(ollama_request)?
        };
        
        // Determinar la URL correcta: si ya contiene un endpoint específico, usarla directamente
        let url = if self.config.api_url.contains("/v1/chat/completions") 
                  || self.config.api_url.contains("/generate") {
            self.config.api_url.clone()
        } else {
            format!("{}/generate", self.config.api_url)
        };
        
        let response = self.client.post(&url)
            .json(&request)
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;
            
        // Obtener la respuesta del formato correcto
        if !response.response.is_empty() {
            // Es una respuesta de Ollama
            Ok(response.response)
        } else if !response.choices.is_empty() && response.choices[0].message.is_some() {
            // Es una respuesta de OpenAI
            Ok(response.choices[0].message.as_ref().unwrap().content.clone())
        } else {
            Err(anyhow::anyhow!("No se pudo obtener respuesta del LLM"))
        }
    }
    
    /// Generar una respuesta utilizando una API compatible con OpenAI
    async fn generate_openai_compatible_response(&self, prompt: &str) -> Result<String> {
        // Estructura para API compatible con OpenAI
        #[derive(Serialize)]
        struct OpenAiRequest {
            model: String,
            messages: Vec<Message>,
            temperature: f32,
            max_tokens: Option<u32>,
        }
        
        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }
        
        let request = OpenAiRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "Eres un asistente de seguridad informática con amplio conocimiento en análisis de comportamiento de procesos y detección de amenazas.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };
        
        let response = self.client.post(&self.config.api_url)
            .json(&request)
            .send()
            .await?
            .json::<Value>()
            .await?;
            
        // Extraer el texto de la respuesta (estructura típica de una API OpenAI)
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .context("No se pudo extraer el contenido de la respuesta")?;
            
        Ok(content.to_string())
    }
} 
