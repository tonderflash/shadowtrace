use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, Duration};

/// Tipo de operación de archivo
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileOperation {
    /// Lectura
    Read,
    /// Escritura
    Write,
    /// Creación
    Create,
    /// Eliminación
    Delete,
    /// Apertura
    Open,
    /// Cierre
    Close,
    /// Renombrado
    Rename { old_path: PathBuf },
    /// Cambio de permisos
    ChangePermissions,
    /// Desconocido
    Unknown,
}

/// Registro de una operación de archivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    /// ID del proceso que realizó la operación
    pub pid: u32,
    /// Ruta del archivo
    pub path: String,
    /// Tipo de operación
    pub operation: FileOperation,
    /// Momento en que ocurrió
    pub timestamp: DateTime<Utc>,
    /// Tamaño de datos transferidos (si aplica)
    pub size: Option<u64>,
    /// Indica si la operación tuvo éxito
    pub success: bool,
}

/// Información sobre un archivo monitoreado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileActivity {
    /// Ruta del archivo
    pub path: PathBuf,
    /// Tipo de operación
    pub operation: FileOperation,
    /// Proceso que realizó la operación (PID)
    pub process_id: Option<u32>,
    /// Tiempo de la operación
    pub timestamp: SystemTime,
    /// Tamaño del archivo (si es conocido)
    pub size: Option<u64>,
}

/// Monitor de operaciones de archivo
pub struct FileMonitor {
    /// Historial de eventos de archivo
    events: Vec<FileEvent>,
    /// Mapa de archivos abiertos por PID
    open_files: HashMap<u32, Vec<String>>,
    /// Rutas bajo monitoreo
    paths: Vec<PathBuf>,
    /// Actividades registradas
    activities: Vec<FileActivity>,
    /// Filtrar por PID
    filter_pid: Option<u32>,
}

impl FileMonitor {
    /// Crear un nuevo monitor de archivos
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            open_files: HashMap::new(),
            paths: Vec::new(),
            activities: Vec::new(),
            filter_pid: None,
        }
    }

    /// Registrar un evento de archivo
    pub fn record_event(&mut self, event: FileEvent) {
        // Actualizar el mapa de archivos abiertos
        match event.operation {
            FileOperation::Open | FileOperation::Create => {
                if event.success {
                    self.open_files
                        .entry(event.pid)
                        .or_insert_with(Vec::new)
                        .push(event.path.clone());
                }
            }
            FileOperation::Close => {
                if let Some(files) = self.open_files.get_mut(&event.pid) {
                    if let Some(pos) = files.iter().position(|p| p == &event.path) {
                        files.remove(pos);
                    }
                }
            }
            _ => {}
        }

        self.events.push(event);
    }

    /// Obtener todos los eventos registrados
    pub fn get_events(&self) -> &[FileEvent] {
        &self.events
    }

    /// Obtener eventos para un proceso específico
    pub fn get_events_for_pid(&self, pid: u32) -> Vec<&FileEvent> {
        self.events.iter().filter(|e| e.pid == pid).collect()
    }

    /// Obtener archivos actualmente abiertos por un proceso
    pub fn get_open_files_for_pid(&self, pid: u32) -> Vec<&String> {
        match self.open_files.get(&pid) {
            Some(files) => files.iter().collect(),
            None => Vec::new(),
        }
    }

    /// Analizar el patrón de acceso a archivos de un proceso
    pub fn analyze_file_pattern(&self, pid: u32) -> Vec<(String, usize)> {
        let events = self.get_events_for_pid(pid);
        
        let mut access_count: HashMap<String, usize> = HashMap::new();
        
        for event in events {
            *access_count.entry(event.path.clone()).or_insert(0) += 1;
        }
        
        // Convertir a vector y ordenar por frecuencia (de mayor a menor)
        let mut result: Vec<(String, usize)> = access_count.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        
        result
    }

    /// Limpiar eventos antiguos (mantener solo los últimos N eventos)
    pub fn clean_old_events(&mut self, keep_count: usize) {
        if self.events.len() > keep_count {
            let to_remove = self.events.len() - keep_count;
            self.events.drain(0..to_remove);
        }
    }
    
    /// Detectar patrones sospechosos de acceso a archivos
    pub fn detect_suspicious_patterns(&self, pid: u32) -> Vec<String> {
        let events = self.get_events_for_pid(pid);
        let mut suspicious = Vec::new();
        
        // Detector de acceso a archivos sensibles
        #[cfg(target_os = "linux")]
        let sensitive_paths = [
            "/etc/passwd", "/etc/shadow", "/etc/ssl", "/etc/ssh", 
            "/var/log", "/.ssh/", "/root/.ssh", "/etc/sudoers",
        ];
        
        #[cfg(target_os = "macos")]
        let sensitive_paths = [
            "/etc/passwd", "/etc/ssl", "/etc/ssh", 
            "/var/log", "/.ssh/", "/Users/root/.ssh", "/etc/sudoers",
            "/private/etc/", "/Library/Keychains/", "/System/Library/",
        ];
        
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        let sensitive_paths = ["/"];
        
        for event in &events {
            for path in &sensitive_paths {
                if event.path.contains(path) {
                    suspicious.push(format!("Acceso a archivo sensible: {}", event.path));
                    break;
                }
            }
        }
        
        // Detector de escritura masiva
        let mut write_count = 0;
        for event in &events {
            if event.operation == FileOperation::Write {
                write_count += 1;
            }
        }
        
        if write_count > 100 {
            suspicious.push(format!("Escritura masiva detectada: {} archivos", write_count));
        }
        
        suspicious
    }

    /// Añadir una ruta para monitorear
    pub fn add_path(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    /// Establecer filtro por PID
    pub fn set_pid_filter(&mut self, pid: Option<u32>) {
        self.filter_pid = pid;
    }

    /// Obtener las actividades registradas
    pub fn get_activities(&self) -> &[FileActivity] {
        &self.activities
    }
    
    /// Añadir una actividad manualmente (para simulación)
    pub fn add_activity(&mut self, activity: FileActivity) {
        self.activities.push(activity);
    }
    
    /// Simular una detección para pruebas
    pub fn simulate_activity(&mut self) {
        // Simulamos algunas operaciones de archivos para probar la UI
        if self.paths.is_empty() {
            // Añadir una ruta por defecto si no hay ninguna
            self.paths.push(PathBuf::from("/tmp"));
        }
        
        let operations = [
            FileOperation::Read,
            FileOperation::Write,
            FileOperation::Create,
        ];
        
        // Simular una actividad con la primera ruta
        if let Some(path) = self.paths.first() {
            let op_index = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs() as usize % operations.len();
                
            self.activities.push(FileActivity {
                path: path.join(format!("file_{}.txt", self.activities.len())),
                operation: operations[op_index].clone(),
                process_id: self.filter_pid,
                timestamp: SystemTime::now(),
                size: Some(1024 * (self.activities.len() as u64 % 10 + 1)),
            });
            
            // Limitar a 100 registros para no consumir demasiada memoria
            if self.activities.len() > 100 {
                self.activities.remove(0);
            }
        }
    }
} 
