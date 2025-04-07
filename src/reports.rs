use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use directories::BaseDirs;

use crate::file_monitor::FileEvent;
use crate::network::NetworkEvent;
use crate::process::ProcessInfo;

/// Niveles de severidad para el reporte
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeverityLevel {
    /// Informativo
    Info,
    /// Advertencia
    Warning,
    /// Alerta
    Alert,
    /// Crítico
    Critical,
}

/// Entrada de reporte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEntry {
    /// Timestamp de la entrada
    pub timestamp: DateTime<Utc>,
    /// Nivel de severidad
    pub severity: SeverityLevel,
    /// Categoría de la entrada
    pub category: String,
    /// Mensaje descriptivo
    pub message: String,
    /// Datos adicionales en formato JSON
    pub data: Option<Value>,
}

/// Reporte completo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// ID del proceso analizado
    pub pid: u32,
    /// Nombre del proceso
    pub process_name: String,
    /// Momento de inicio del análisis
    pub analysis_start: DateTime<Utc>,
    /// Momento de fin del análisis
    pub analysis_end: DateTime<Utc>,
    /// Información del proceso
    pub process_info: Option<ProcessInfo>,
    /// Eventos de archivo detectados
    pub file_events: Vec<FileEvent>,
    /// Eventos de red detectados
    pub network_events: Vec<NetworkEvent>,
    /// Análisis del LLM
    pub llm_analysis: Option<String>,
    /// Entradas de reporte
    pub entries: Vec<ReportEntry>,
}

impl Report {
    /// Crear un nuevo reporte para un proceso
    pub fn new(pid: u32, process_name: String) -> Self {
        let now = Utc::now();
        
        Self {
            pid,
            process_name,
            analysis_start: now,
            analysis_end: now,
            process_info: None,
            file_events: Vec::new(),
            network_events: Vec::new(),
            llm_analysis: None,
            entries: Vec::new(),
        }
    }
    
    /// Actualizar el momento de fin del análisis
    pub fn update_end_time(&mut self) {
        self.analysis_end = Utc::now();
    }
    
    /// Establecer la información del proceso
    pub fn set_process_info(&mut self, process_info: ProcessInfo) {
        self.process_info = Some(process_info);
    }
    
    /// Agregar un evento de archivo
    pub fn add_file_event(&mut self, event: FileEvent) {
        self.file_events.push(event);
    }
    
    /// Agregar un evento de red
    pub fn add_network_event(&mut self, event: NetworkEvent) {
        self.network_events.push(event);
    }
    
    /// Establecer el análisis del LLM
    pub fn set_llm_analysis(&mut self, analysis: String) {
        self.llm_analysis = Some(analysis);
    }
    
    /// Agregar una entrada al reporte
    pub fn add_entry(&mut self, entry: ReportEntry) {
        self.entries.push(entry);
    }
    
    /// Agregar una entrada informativa
    pub fn add_info(&mut self, category: &str, message: &str, data: Option<Value>) {
        self.add_entry(ReportEntry {
            timestamp: Utc::now(),
            severity: SeverityLevel::Info,
            category: category.to_string(),
            message: message.to_string(),
            data,
        });
    }
    
    /// Agregar una entrada de advertencia
    pub fn add_warning(&mut self, category: &str, message: &str, data: Option<Value>) {
        self.add_entry(ReportEntry {
            timestamp: Utc::now(),
            severity: SeverityLevel::Warning,
            category: category.to_string(),
            message: message.to_string(),
            data,
        });
    }
    
    /// Agregar una entrada de alerta
    pub fn add_alert(&mut self, category: &str, message: &str, data: Option<Value>) {
        self.add_entry(ReportEntry {
            timestamp: Utc::now(),
            severity: SeverityLevel::Alert,
            category: category.to_string(),
            message: message.to_string(),
            data,
        });
    }
    
    /// Agregar una entrada crítica
    pub fn add_critical(&mut self, category: &str, message: &str, data: Option<Value>) {
        self.add_entry(ReportEntry {
            timestamp: Utc::now(),
            severity: SeverityLevel::Critical,
            category: category.to_string(),
            message: message.to_string(),
            data,
        });
    }
    
    /// Guardar el reporte en formato JSON
    pub fn save_json<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
    
    /// Generar un reporte en formato Markdown
    pub fn generate_markdown(&self) -> String {
        let local_start = DateTime::<Local>::from(self.analysis_start);
        let local_end = DateTime::<Local>::from(self.analysis_end);
        
        let mut md = String::new();
        
        // Título y encabezado
        md.push_str(&format!("# Reporte ShadowTrace: {} (PID: {})\n\n", self.process_name, self.pid));
        md.push_str(&format!("**Generado el:** {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // Resumen
        md.push_str("## Resumen\n\n");
        md.push_str(&format!("- **Proceso:** {} (PID: {})\n", self.process_name, self.pid));
        md.push_str(&format!("- **Análisis iniciado:** {}\n", local_start.format("%Y-%m-%d %H:%M:%S")));
        md.push_str(&format!("- **Análisis finalizado:** {}\n", local_end.format("%Y-%m-%d %H:%M:%S")));
        md.push_str(&format!("- **Duración:** {} segundos\n", (self.analysis_end - self.analysis_start).num_seconds()));
        md.push_str(&format!("- **Eventos de archivos:** {}\n", self.file_events.len()));
        md.push_str(&format!("- **Eventos de red:** {}\n", self.network_events.len()));
        md.push_str(&format!("- **Entradas de reporte:** {}\n\n", self.entries.len()));
        
        // Información del proceso
        if let Some(info) = &self.process_info {
            md.push_str("## Información del Proceso\n\n");
            md.push_str(&format!("- **Nombre:** {}\n", info.name));
            if let Some(path) = &info.path {
                md.push_str(&format!("- **Ruta ejecutable:** {}\n", path));
            }
            if let Some(cmd) = &info.cmd_line {
                md.push_str(&format!("- **Línea de comandos:** {}\n", cmd.join(" ")));
            }
            md.push_str(&format!("- **Uso de CPU:** {:.2}%\n", info.cpu_usage));
            md.push_str(&format!("- **Uso de memoria:** {} KB\n", info.memory_usage));
            md.push_str(&format!("- **Tiempo de inicio:** {}\n", 
                DateTime::<Local>::from(info.start_time).format("%Y-%m-%d %H:%M:%S")));
            if !info.children.is_empty() {
                md.push_str(&format!("- **Procesos hijos:** {}\n", info.children.len()));
                for child_pid in &info.children {
                    md.push_str(&format!("  - PID: {}\n", child_pid));
                }
            }
            md.push_str("\n");
        }
        
        // Análisis del LLM
        if let Some(analysis) = &self.llm_analysis {
            md.push_str("## Análisis de IA\n\n");
            md.push_str(&format!("{}\n\n", analysis));
        }
        
        // Alertas y advertencias
        let alerts = self.entries.iter()
            .filter(|e| e.severity == SeverityLevel::Alert || e.severity == SeverityLevel::Critical)
            .collect::<Vec<_>>();
            
        if !alerts.is_empty() {
            md.push_str("## ⚠️ Alertas Detectadas\n\n");
            for alert in alerts {
                let severity_marker = match alert.severity {
                    SeverityLevel::Critical => "🔴 CRÍTICO",
                    SeverityLevel::Alert => "🟠 ALERTA",
                    _ => "",
                };
                md.push_str(&format!("### {} - {}\n\n", severity_marker, alert.category));
                md.push_str(&format!("{}\n\n", alert.message));
                if let Some(data) = &alert.data {
                    md.push_str("```json\n");
                    md.push_str(&serde_json::to_string_pretty(data).unwrap_or_default());
                    md.push_str("\n```\n\n");
                }
            }
        }
        
        // Resumen de acceso a archivos
        if !self.file_events.is_empty() {
            md.push_str("## Actividad de Archivos\n\n");
            
            // Agrupar por operación
            let mut operations: HashMap<String, usize> = HashMap::new();
            for event in &self.file_events {
                *operations.entry(format!("{:?}", event.operation)).or_insert(0) += 1;
            }
            
            md.push_str("### Operaciones por tipo\n\n");
            for (op, count) in operations {
                md.push_str(&format!("- {}: {} operaciones\n", op, count));
            }
            md.push_str("\n");
            
            // Top 10 archivos más accedidos
            let mut file_access: HashMap<&String, usize> = HashMap::new();
            for event in &self.file_events {
                *file_access.entry(&event.path).or_insert(0) += 1;
            }
            
            let mut file_access_vec: Vec<(&String, usize)> = file_access.into_iter().collect();
            file_access_vec.sort_by(|a, b| b.1.cmp(&a.1));
            
            md.push_str("### Top archivos accedidos\n\n");
            for (i, (path, count)) in file_access_vec.iter().take(10).enumerate() {
                md.push_str(&format!("{}. `{}` - {} accesos\n", i+1, path, count));
            }
            md.push_str("\n");
        }
        
        // Resumen de conexiones de red
        if !self.network_events.is_empty() {
            md.push_str("## Actividad de Red\n\n");
            
            // Contar conexiones por dirección
            let mut inbound = 0;
            let mut outbound = 0;
            for event in &self.network_events {
                match event.direction {
                    crate::network::Direction::Inbound => inbound += 1,
                    crate::network::Direction::Outbound => outbound += 1,
                }
            }
            
            md.push_str(&format!("- Conexiones entrantes: {}\n", inbound));
            md.push_str(&format!("- Conexiones salientes: {}\n\n", outbound));
            
            // Top 10 destinos
            let mut destinations: HashMap<String, usize> = HashMap::new();
            for event in &self.network_events {
                if let Some(addr) = &event.remote_addr {
                    *destinations.entry(addr.to_string()).or_insert(0) += 1;
                }
            }
            
            let mut dest_vec: Vec<(String, usize)> = destinations.into_iter().collect();
            dest_vec.sort_by(|a, b| b.1.cmp(&a.1));
            
            if !dest_vec.is_empty() {
                md.push_str("### Top destinos de conexión\n\n");
                for (i, (addr, count)) in dest_vec.iter().take(10).enumerate() {
                    md.push_str(&format!("{}. `{}` - {} conexiones\n", i+1, addr, count));
                }
                md.push_str("\n");
            }
        }
        
        // Registro cronológico de eventos
        md.push_str("## Registro Cronológico\n\n");
        md.push_str("| Tiempo | Severidad | Categoría | Mensaje |\n");
        md.push_str("|--------|-----------|-----------|--------|\n");
        
        for entry in &self.entries {
            let local_time = DateTime::<Local>::from(entry.timestamp);
            let severity = match entry.severity {
                SeverityLevel::Info => "INFO",
                SeverityLevel::Warning => "⚠️ WARN",
                SeverityLevel::Alert => "🔶 ALERTA",
                SeverityLevel::Critical => "🔴 CRÍTICO",
            };
            
            let message = entry.message.replace("|", "\\|");  // Escapar caracteres pipe para Markdown
            
            md.push_str(&format!("| {} | {} | {} | {} |\n",
                local_time.format("%H:%M:%S"),
                severity,
                entry.category,
                message,
            ));
        }
        
        md
    }
    
    /// Guardar el reporte en formato Markdown
    pub fn save_markdown<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let markdown = self.generate_markdown();
        let mut file = File::create(path)?;
        file.write_all(markdown.as_bytes())?;
        Ok(())
    }
    
    /// Generar nombre de archivo para el reporte basado en tiempo y proceso
    pub fn generate_filename(&self, extension: &str) -> String {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        format!("shadowtrace_{}_pid{}_{}.{}", 
            self.process_name.replace(" ", "_"), 
            self.pid, 
            timestamp,
            extension)
    }
    
    /// Guardar en directorio por defecto
    pub fn save_to_default_dir(&self) -> Result<(PathBuf, PathBuf)> {
        // Crear directorio de reportes si no existe
        let base_dir = if let Some(base_dirs) = BaseDirs::new() {
            let home_dir = base_dirs.home_dir();
            home_dir.join(".shadowtrace").join("reports")
        } else {
            return Err(anyhow::anyhow!("No se pudo determinar el directorio home"));
        };
            
        fs::create_dir_all(&base_dir)?;
        
        // Generar nombres de archivo
        let json_filename = self.generate_filename("json");
        let md_filename = self.generate_filename("md");
        
        let json_path = base_dir.join(&json_filename);
        let md_path = base_dir.join(&md_filename);
        
        // Guardar reportes
        self.save_json(&json_path)?;
        self.save_markdown(&md_path)?;
        
        Ok((json_path, md_path))
    }
} 
