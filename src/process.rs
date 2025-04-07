use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

/// Estructura que representa un proceso monitorizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// ID del proceso
    pub pid: u32,
    /// Nombre del proceso
    pub name: String,
    /// Ruta ejecutable
    pub path: Option<String>,
    /// Argumentos de línea de comandos
    pub cmd_line: Option<Vec<String>>,
    /// Usuario que ejecuta el proceso
    pub user: Option<String>,
    /// Uso de CPU
    pub cpu_usage: f32,
    /// Uso de memoria (KB)
    pub memory_usage: u64,
    /// Tiempo de inicio
    pub start_time: DateTime<Utc>,
    /// Procesos hijos
    pub children: Vec<u32>,
}

/// Estructura para monitorizar procesos
pub struct ProcessMonitor {
    system: System,
}

impl ProcessMonitor {
    /// Crear un nuevo monitor de procesos
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self { system }
    }

    /// Refrescar la información del sistema
    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    /// Obtener información sobre un proceso específico por PID
    pub fn get_process_by_pid(&mut self, pid: u32) -> Option<ProcessInfo> {
        let pid = Pid::from_u32(pid);
        
        self.system.refresh_process(pid);
        
        self.system.process(pid).map(|process| {
            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                path: Some(process.exe().to_string_lossy().to_string()),
                cmd_line: Some(process.cmd().iter().take(5).map(|s| s.to_string()).collect()),
                user: None, // No disponible directamente en sysinfo
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                start_time: chrono::DateTime::from_timestamp(process.start_time() as i64, 0)
                    .unwrap_or_else(|| Utc::now()),
                children: Vec::new(),
            }
        })
    }

    /// Obtener todos los procesos activos
    pub fn get_all_processes(&mut self) -> Vec<ProcessInfo> {
        self.system.refresh_processes();
        
        self.system
            .processes()
            .iter()
            .take(100)
            .map(|(pid, process)| {
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    path: None,
                    cmd_line: None,
                    user: None,
                    cpu_usage: process.cpu_usage(),
                    memory_usage: process.memory(),
                    start_time: Utc::now(),
                    children: Vec::new(),
                }
            })
            .collect()
    }

    /// Buscar procesos por nombre
    pub fn find_process_by_name(&mut self, name: &str) -> Vec<ProcessInfo> {
        self.system.refresh_all();
        
        self.system
            .processes()
            .iter()
            .filter(|(_, process)| process.name().to_lowercase().contains(&name.to_lowercase()))
            .map(|(pid, process)| {
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    path: Some(process.exe().to_string_lossy().to_string()),
                    cmd_line: Some(process.cmd().iter().map(|s| s.to_string()).collect()),
                    user: None,
                    cpu_usage: process.cpu_usage(),
                    memory_usage: process.memory(),
                    start_time: chrono::DateTime::from_timestamp(process.start_time() as i64, 0)
                        .unwrap_or_else(|| Utc::now()),
                    children: Vec::new(),
                }
            })
            .collect()
    }
} 
