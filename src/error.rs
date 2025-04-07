use thiserror::Error;

/// Errores específicos de la aplicación
#[derive(Error, Debug)]
pub enum AppError {
    /// Error al acceder a un proceso
    #[error("Error al acceder al proceso: {0}")]
    ProcessAccessError(String),
    
    /// Error al monitorear archivos
    #[error("Error al monitorear archivos: {0}")]
    FileMonitorError(String),
    
    /// Error al monitorear red
    #[error("Error al monitorear red: {0}")]
    NetworkMonitorError(String),
    
    /// Error de comunicación con LLM
    #[error("Error de comunicación con LLM: {0}")]
    LlmCommunicationError(String),
    
    /// Error al generar reporte
    #[error("Error al generar reporte: {0}")]
    ReportGenerationError(String),
    
    /// Error al guardar reporte
    #[error("Error al guardar reporte: {0}")]
    ReportSaveError(String),
    
    /// Error de configuración
    #[error("Error de configuración: {0}")]
    ConfigurationError(String),
    
    /// Error genérico
    #[error("Error: {0}")]
    GenericError(String),
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::GenericError(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::GenericError(error.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError::LlmCommunicationError(error.to_string())
    }
} 
