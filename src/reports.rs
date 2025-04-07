use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use directories::BaseDirs;
use std::time::{SystemTime, Duration};

use crate::file_monitor::FileEvent;
use crate::network::NetworkEvent;
use crate::process::ProcessInfo;
use crate::file_monitor::FileActivity;

/// Estado de un reporte
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportStatus {
    /// En progreso
    InProgress,
    /// Completado
    Completed,
    /// Con advertencias
    Warning,
    /// Con errores
    Error,
}

/// Niveles de severidad para el reporte
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeverityLevel {
    /// Informativo
    Info,
    /// Advertencia
    Warning,
    /// Error
    Error,
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

/// Hallazgo o anomalía detectada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Título del hallazgo
    pub title: String,
    /// Descripción
    pub description: String,
    /// Severidad
    pub severity: SeverityLevel,
    /// Recomendación
    pub recommendation: Option<String>,
    /// Recursos afectados
    pub affected_resources: Vec<String>,
    /// Timestamp
    pub timestamp: SystemTime,
}

/// Reporte de análisis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// ID único
    pub id: String,
    /// Título
    pub title: String,
    /// Timestamp de creación
    pub created_at: SystemTime,
    /// Estado actual
    pub status: ReportStatus,
    /// Duración del análisis
    pub duration: Duration,
    /// Procesos analizados
    pub processes: Vec<ProcessInfo>,
    /// Actividad de archivos
    pub file_activities: Vec<FileActivity>,
    /// Eventos de red
    pub network_events: Vec<NetworkEvent>,
    /// Hallazgos detectados
    pub findings: Vec<Finding>,
    /// Resumen
    pub summary: String,
}

impl Report {
    /// Crear un nuevo reporte
    pub fn new(title: &str) -> Self {
        let now = SystemTime::now();
        Self {
            id: format!("report_{}", now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()),
            title: title.to_string(),
            created_at: now,
            status: ReportStatus::InProgress,
            duration: Duration::from_secs(0),
            processes: Vec::new(),
            file_activities: Vec::new(),
            network_events: Vec::new(),
            findings: Vec::new(),
            summary: String::new(),
        }
    }

    /// Añadir un proceso
    pub fn add_process(&mut self, process: ProcessInfo) {
        self.processes.push(process);
    }

    /// Añadir actividad de archivo
    pub fn add_file_activity(&mut self, activity: FileActivity) {
        self.file_activities.push(activity);
    }

    /// Añadir evento de red
    pub fn add_network_event(&mut self, event: NetworkEvent) {
        self.network_events.push(event);
    }

    /// Añadir un hallazgo
    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// Completar el reporte
    pub fn complete(&mut self, summary: &str) {
        self.status = ReportStatus::Completed;
        self.duration = SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::from_secs(0));
        self.summary = summary.to_string();
    }

    /// Establecer estado
    pub fn set_status(&mut self, status: ReportStatus) {
        self.status = status;
    }

    /// Crear un nuevo reporte para un proceso
    pub fn new_for_process(pid: u32, process_name: String) -> Self {
        let now = Utc::now();
        
        Self {
            id: format!("report_{}", now.timestamp()),
            title: format!("Análisis de {}", process_name),
            created_at: now.into(),
            status: ReportStatus::InProgress,
            duration: Duration::from_secs(0),
            processes: vec![ProcessInfo {
                pid,
                name: process_name,
                path: None,
                cmd_line: None,
                cpu_usage: 0.0,
                memory_usage: 0,
                start_time: now.into(),
                children: Vec::new(),
                user: None,
            }],
            file_activities: Vec::new(),
            network_events: Vec::new(),
            findings: Vec::new(),
            summary: String::new(),
        }
    }
    
    /// Actualizar el momento de fin del análisis
    pub fn update_end_time(&mut self) {
        self.duration = SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::from_secs(0));
    }
    
    /// Establecer la información del proceso
    pub fn set_process_info(&mut self, process_info: ProcessInfo) {
        self.processes[0] = process_info;
    }
    
    /// Agregar una entrada al reporte
    pub fn add_entry(&mut self, entry: ReportEntry) {
        self.findings.push(Finding {
            title: entry.category.clone(),
            description: entry.message.clone(),
            severity: entry.severity,
            recommendation: None,
            affected_resources: Vec::new(),
            timestamp: SystemTime::now(),
        });
    }
    
    /// Agregar una entrada informativa
    pub fn add_info(&mut self, category: &str, message: &str, data: Option<Value>) {
        self.add_entry(ReportEntry {
            timestamp: Utc::now().into(),
            severity: SeverityLevel::Info,
            category: category.to_string(),
            message: message.to_string(),
            data,
        });
    }
    
    /// Agregar una entrada de advertencia
    pub fn add_warning(&mut self, category: &str, message: &str, data: Option<Value>) {
        self.add_entry(ReportEntry {
            timestamp: Utc::now().into(),
            severity: SeverityLevel::Warning,
            category: category.to_string(),
            message: message.to_string(),
            data,
        });
    }
    
    /// Agregar una entrada de alerta
    pub fn add_alert(&mut self, category: &str, message: &str, data: Option<Value>) {
        self.add_entry(ReportEntry {
            timestamp: Utc::now().into(),
            severity: SeverityLevel::Critical,
            category: category.to_string(),
            message: message.to_string(),
            data,
        });
    }
    
    /// Agregar una entrada crítica
    pub fn add_critical(&mut self, category: &str, message: &str, data: Option<Value>) {
        self.add_entry(ReportEntry {
            timestamp: Utc::now().into(),
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
        let mut md = String::new();
        
        // Título y encabezado
        md.push_str(&format!("# Reporte ShadowTrace: {} (ID: {})\n\n", self.title, self.id));
        md.push_str(&format!("**Generado el:** {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // Resumen
        md.push_str("## Resumen\n\n");
        md.push_str(&format!("- **Proceso:** {}\n", self.processes[0].name));
        md.push_str(&format!("- **Análisis iniciado:** {}\n", 
            DateTime::<Local>::from(self.created_at).format("%Y-%m-%d %H:%M:%S")));
        md.push_str(&format!("- **Análisis finalizado:** {}\n", 
            DateTime::<Local>::from(self.created_at + self.duration).format("%Y-%m-%d %H:%M:%S")));
        md.push_str(&format!("- **Duración:** {} segundos\n", self.duration.as_secs()));
        md.push_str(&format!("- **Hallazgos detectados:** {}\n\n", self.findings.len()));
        
        // Información del proceso
        md.push_str("## Información del Proceso\n\n");
        md.push_str(&format!("- **Nombre:** {}\n", self.processes[0].name));
        if let Some(path) = &self.processes[0].path {
            md.push_str(&format!("- **Ruta ejecutable:** {}\n", path));
        }
        if let Some(cmd) = &self.processes[0].cmd_line {
            md.push_str(&format!("- **Línea de comandos:** {}\n", cmd.join(" ")));
        }
        md.push_str(&format!("- **Uso de CPU:** {:.2}%\n", self.processes[0].cpu_usage));
        md.push_str(&format!("- **Uso de memoria:** {} KB\n", self.processes[0].memory_usage));
        md.push_str(&format!("- **Tiempo de inicio:** {}\n", 
            DateTime::<Local>::from(self.processes[0].start_time).format("%Y-%m-%d %H:%M:%S")));
        if !self.processes[0].children.is_empty() {
            md.push_str(&format!("- **Procesos hijos:** {}\n", self.processes[0].children.len()));
            for child_pid in &self.processes[0].children {
                md.push_str(&format!("  - PID: {}\n", child_pid));
            }
        }
        md.push_str("\n");
        
        // Hallazgos detectados
        if !self.findings.is_empty() {
            md.push_str("## Hallazgos Detectados\n\n");
            for finding in &self.findings {
                let severity_marker = match finding.severity {
                    SeverityLevel::Critical => "🔴 CRÍTICO",
                    SeverityLevel::Error => "🟠 ERROR",
                    _ => "",
                };
                md.push_str(&format!("### {} - {}\n\n", severity_marker, finding.title));
                md.push_str(&format!("{}\n\n", finding.description));
                if let Some(recommendation) = &finding.recommendation {
                    md.push_str(&format!("**Recomendación:** {}\n\n", recommendation));
                }
                if !finding.affected_resources.is_empty() {
                    md.push_str("**Recursos afectados:**\n");
                    for resource in &finding.affected_resources {
                        md.push_str(&format!("- {}\n", resource));
                    }
                    md.push_str("\n");
                }
            }
        }
        
        // Resumen de acceso a archivos
        if !self.file_activities.is_empty() {
            md.push_str("## Actividad de Archivos\n\n");
            
            // Agrupar por operación
            let mut operations: HashMap<String, usize> = HashMap::new();
            for activity in &self.file_activities {
                *operations.entry(format!("{:?}", activity.operation)).or_insert(0) += 1;
            }
            
            md.push_str("### Operaciones por tipo\n\n");
            for (op, count) in operations {
                md.push_str(&format!("- {}: {} operaciones\n", op, count));
            }
            md.push_str("\n");
            
            // Top 10 archivos más accedidos
            let mut file_access: HashMap<String, usize> = HashMap::new();
            for activity in &self.file_activities {
                let path_str = activity.path.to_string_lossy().to_string();
                *file_access.entry(path_str).or_insert(0) += 1;
            }
            
            let mut file_access_vec: Vec<(String, usize)> = file_access.into_iter().collect();
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
        
        for finding in &self.findings {
            let local_time = DateTime::<Local>::from(finding.timestamp);
            let severity = match finding.severity {
                SeverityLevel::Info => "INFO",
                SeverityLevel::Warning => "⚠️ WARN",
                SeverityLevel::Error => "🟠 ERROR",
                SeverityLevel::Critical => "🔴 CRÍTICO",
            };
            
            let message = finding.title.replace("|", "\\|");  // Escapar caracteres pipe para Markdown
            
            md.push_str(&format!("| {} | {} | {} | {} |\n",
                local_time.format("%H:%M:%S"),
                severity,
                finding.title,
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
        format!("shadowtrace_{}_id{}_{}.{}", 
            self.title.replace(" ", "_"), 
            self.id, 
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

    /// Generar un reporte de ejemplo para propósitos de demo
    pub fn demo() -> Self {
        let now = Utc::now();
        
        Self {
            id: format!("report_{}", now.timestamp()),
            title: String::from("Reporte de demostración"),
            created_at: now.into(),
            status: ReportStatus::Completed,
            duration: Duration::from_secs(60),
            processes: vec![ProcessInfo {
                pid: 1234,
                name: String::from("demo_process"),
                path: Some(String::from("/usr/bin/demo_process")),
                cmd_line: Some(vec![String::from("/usr/bin/demo_process"), String::from("--arg1"), String::from("--arg2")]),
                cpu_usage: 5.2,
                memory_usage: 128,
                start_time: now.into(),
                children: Vec::new(),
                user: Some(String::from("usuario")),
            }],
            file_activities: Vec::new(),
            network_events: Vec::new(),
            findings: Vec::new(),
            summary: String::from("Este es un reporte de demostración generado automáticamente."),
        }
    }
} 
