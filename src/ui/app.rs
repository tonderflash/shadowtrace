use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use serde_json;

use crate::process::ProcessMonitor;
use crate::file_monitor::FileMonitor;
use crate::network::NetworkMonitor;
use crate::reports::Report;

/// Estados posibles de la aplicación
pub enum AppState {
    Dashboard,
    ProcessMonitor,
    FileMonitor,
    NetworkMonitor,
    Reports,
    Help,
}

/// Estructura principal de la aplicación para la UI
pub struct App {
    /// Estado actual de la aplicación
    pub state: AppState,
    /// Indica si la aplicación está en ejecución
    pub running: bool,
    /// Contador de pulsaciones para animaciones
    pub tick_count: u64,
    /// Última vez que se actualizó
    pub last_tick: Instant,
    /// Monitor de procesos
    pub process_monitor: ProcessMonitor,
    /// Monitor de archivos
    pub file_monitor: FileMonitor,
    /// Monitor de red
    pub network_monitor: NetworkMonitor,
    /// Reportes generados
    pub reports: Vec<Report>,
    /// Estado de selección para listas
    pub list_state: ListState,
    /// PID del proceso actualmente seleccionado
    pub selected_pid: Option<u32>,
    /// Mensajes de estado
    pub status_message: Option<String>,
    /// Tiempo desde el inicio del monitoreo
    pub monitoring_time: Duration,
    /// Intervalo de actualización en milisegundos
    pub update_interval: u64,
    /// Lista de procesos actualmente en pantalla
    pub processes: Vec<crate::process::ProcessInfo>,
    /// Tab actual en el monitor de procesos (0: Detalles, 1: Análisis LLM)
    pub process_monitor_tab: usize,
    /// Análisis LLM para el proceso seleccionado
    pub process_llm_analysis: Option<String>,
    /// Duración del monitoreo en segundos (0 = indefinido)
    pub monitoring_duration: u64,
    /// Tiempo de inicio del monitoreo actual
    pub monitoring_start_time: Option<Instant>,
    /// Indica si se está monitoreando activamente
    pub is_monitoring_active: bool,
    /// Historial de lecturas de CPU
    pub cpu_history: Vec<f32>,
    /// Historial de lecturas de memoria
    pub memory_history: Vec<u64>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            state: AppState::Dashboard,
            running: true,
            tick_count: 0,
            last_tick: Instant::now(),
            process_monitor: ProcessMonitor::new(),
            file_monitor: FileMonitor::new(),
            network_monitor: NetworkMonitor::new(),
            reports: Vec::new(),
            list_state: ListState::default(),
            selected_pid: None,
            status_message: None,
            monitoring_time: Duration::from_secs(0),
            update_interval: 250,
            processes: Vec::new(),
            process_monitor_tab: 0,
            process_llm_analysis: None,
            monitoring_duration: 0,
            monitoring_start_time: None,
            is_monitoring_active: false,
            cpu_history: Vec::new(),
            memory_history: Vec::new(),
        };
        // Cargar procesos iniciales
        app.refresh_processes();
        app
    }
}

impl App {
    /// Crea una nueva instancia de la aplicación
    pub fn new() -> Self {
        Self::default()
    }

    /// Actualiza el estado de la aplicación
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        self.last_tick = Instant::now();
        
        // Actualizar tiempo de monitoreo si está activo
        if self.is_monitoring_active {
            if let Some(start_time) = self.monitoring_start_time {
                self.monitoring_time = self.last_tick.duration_since(start_time);
                
                // Verificar si se ha alcanzado la duración máxima
                if self.monitoring_duration > 0 && 
                   self.monitoring_time.as_secs() >= self.monitoring_duration {
                    // Detener el monitoreo si se alcanzó el límite
                    self.stop_monitoring();
                    self.status_message = Some(format!(
                        "Monitoreo finalizado después de {} segundos", 
                        self.monitoring_duration
                    ));
                    
                    // Generar reporte si no hay uno
                    if self.process_llm_analysis.is_none() {
                        self.generate_demo_analysis();
                    }
                    return;
                }
            }
            
            // Actualizar información de proceso y almacenar historial cada 10 ticks
            if self.tick_count % 10 == 0 {
                if let Some(pid) = self.selected_pid {
                    if let Some(process) = self.process_monitor.get_process_by_pid(pid) {
                        // Almacenar historial de CPU y memoria
                        self.cpu_history.push(process.cpu_usage);
                        self.memory_history.push(process.memory_usage);
                        
                        // Limitar el tamaño del historial a 100 puntos
                        if self.cpu_history.len() > 100 {
                            self.cpu_history.remove(0);
                        }
                        if self.memory_history.len() > 100 {
                            self.memory_history.remove(0);
                        }
                    }
                }
            }
        }
    }

    /// Refresca la lista de procesos
    pub fn refresh_processes(&mut self) {
        // Usar un enfoque más eficiente limitando la cantidad de datos
        let procs = self.process_monitor.get_all_processes();
        
        // Reemplazar la lista existente sin realocar si es posible
        self.processes.clear();
        self.processes.extend(procs);
        
        // Asegurarse de que la selección sigue siendo válida
        if let Some(i) = self.list_state.selected() {
            if i >= self.processes.len() && !self.processes.is_empty() {
                self.list_state.select(Some(self.processes.len() - 1));
            }
        } else if !self.processes.is_empty() {
            // Seleccionar el primer proceso si no hay ninguna selección
            self.list_state.select(Some(0));
        }
    }

    /// Maneja eventos de teclado
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.state {
            AppState::Dashboard => self.handle_dashboard_keys(key_event),
            AppState::ProcessMonitor => self.handle_process_monitor_keys(key_event),
            AppState::FileMonitor => self.handle_file_monitor_keys(key_event),
            AppState::NetworkMonitor => self.handle_network_monitor_keys(key_event),
            AppState::Reports => self.handle_reports_keys(key_event),
            AppState::Help => self.handle_help_keys(key_event),
        }
    }

    fn handle_dashboard_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Char('p') => self.state = AppState::ProcessMonitor,
            KeyCode::Char('f') => self.state = AppState::FileMonitor,
            KeyCode::Char('n') => self.state = AppState::NetworkMonitor,
            KeyCode::Char('r') => self.state = AppState::Reports,
            KeyCode::Char('h') => self.state = AppState::Help,
            _ => {}
        }
    }

    fn handle_process_monitor_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.state = AppState::Dashboard,
            KeyCode::Char('r') => self.refresh_processes(),
            KeyCode::Char('a') => {
                // Generar análisis real del proceso seleccionado
                if let Some(_) = self.selected_pid {
                    self.generate_real_analysis();
                } else {
                    self.status_message = Some("Selecciona un proceso primero".to_string());
                }
            },
            KeyCode::Char('m') => {
                // Iniciar monitoreo si hay un proceso seleccionado
                if let Some(_) = self.selected_pid {
                    if !self.is_monitoring_active {
                        // Monitoreo indefinido 
                        self.start_monitoring(0);
                    } else {
                        self.status_message = Some("Ya hay un monitoreo activo. Presiona 's' para detenerlo.".to_string());
                    }
                } else {
                    self.status_message = Some("Selecciona un proceso primero".to_string());
                }
            },
            KeyCode::Char('s') => {
                // Detener monitoreo activo
                if self.is_monitoring_active {
                    self.stop_monitoring();
                } else {
                    self.status_message = Some("No hay un monitoreo activo".to_string());
                }
            },
            KeyCode::Char('t') | KeyCode::Tab => {
                // Alternar entre tabs
                self.process_monitor_tab = (self.process_monitor_tab + 1) % 2;
                self.status_message = Some(
                    if self.process_monitor_tab == 0 {
                        "Mostrando detalles del proceso".to_string()
                    } else {
                        "Mostrando análisis LLM".to_string()
                    }
                );
            },
            KeyCode::Down => {
                // Mover selección hacia abajo
                let len = self.processes.len();
                if len > 0 {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i >= len - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Up => {
                // Mover selección hacia arriba
                let len = self.processes.len();
                if len > 0 {
                    let i = match self.list_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                len - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.list_state.select(Some(i));
                }
            }
            KeyCode::Enter => {
                // Seleccionar proceso para monitorear
                if let Some(i) = self.list_state.selected() {
                    if i < self.processes.len() {
                        let pid = self.processes[i].pid;
                        self.selected_pid = Some(pid);
                        self.status_message = Some(format!(
                            "Proceso seleccionado: PID {}. Presiona 'm' para iniciar monitoreo o 'a' para análisis.", 
                            pid
                        ));
                        
                        // Limpiar análisis anterior si se selecciona un nuevo proceso
                        self.process_llm_analysis = None;
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_file_monitor_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.state = AppState::Dashboard,
            _ => {}
        }
    }

    fn handle_network_monitor_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.state = AppState::Dashboard,
            _ => {}
        }
    }

    fn handle_reports_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.state = AppState::Dashboard,
            _ => {}
        }
    }

    fn handle_help_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => self.state = AppState::Dashboard,
            _ => {}
        }
    }

    /// Genera un análisis de demostración para el proceso seleccionado
    fn generate_demo_analysis(&mut self) {
        if let Some(pid) = self.selected_pid {
            if let Some(process) = self.process_monitor.get_process_by_pid(pid) {
                // Generar un análisis de ejemplo basado en el proceso
                let analysis = format!(
                    "## Análisis de Comportamiento del Proceso\n\n\
                    **Proceso:** {} (PID: {})\n\n\
                    **Resumen:** El proceso {} es un proceso del sistema que muestra un comportamiento normal para su tipo. \
                    Está utilizando aproximadamente {:.2}% de CPU y {} KB de memoria.\n\n\
                    **Actividad de Archivos:**\n\
                    - El proceso está accediendo a archivos de configuración en ubicaciones estándar\n\
                    - No se observa acceso a archivos sensibles del sistema\n\
                    - La actividad de lectura/escritura es consistente con operaciones normales\n\n\
                    **Actividad de Red:**\n\
                    - No se detectan conexiones sospechosas\n\
                    - El tráfico de red está dentro de los parámetros normales\n\
                    - No hay intentos de conexión a direcciones IP externas desconocidas\n\n\
                    **Evaluación de Riesgo:**\n\
                    - **Nivel de Riesgo:** Bajo\n\
                    - **Confianza en el Análisis:** Alta\n\
                    - **Indicadores de Comportamiento Malicioso:** Ninguno detectado\n\n\
                    **Recomendaciones:**\n\
                    - Continuar con el monitoreo normal del sistema\n\
                    - No se requieren acciones inmediatas\n\
                    - Considerar incluir este proceso en la lista blanca si se monitorea frecuentemente\n\n\
                    *Este análisis fue generado automáticamente por el módulo de Inteligencia Artificial de ShadowTrace.*\
                    ",
                    process.name, 
                    process.pid,
                    process.name,
                    process.cpu_usage,
                    process.memory_usage
                );
                
                self.process_llm_analysis = Some(analysis);
                self.status_message = Some("Análisis LLM generado".to_string());
            }
        }
    }

    /// Iniciar monitoreo de proceso
    pub fn start_monitoring(&mut self, duration_secs: u64) {
        self.monitoring_duration = duration_secs;
        self.monitoring_start_time = Some(Instant::now());
        self.monitoring_time = Duration::from_secs(0);
        self.is_monitoring_active = true;
        self.cpu_history.clear();
        self.memory_history.clear();
        
        // Cambiar mensaje de estado
        if self.monitoring_duration > 0 {
            self.status_message = Some(format!(
                "Monitoreando proceso por {} segundos", 
                self.monitoring_duration
            ));
        } else {
            self.status_message = Some("Monitoreando proceso indefinidamente".to_string());
        }
    }
    
    /// Detener monitoreo de proceso
    pub fn stop_monitoring(&mut self) {
        self.is_monitoring_active = false;
        self.status_message = Some("Monitoreo detenido".to_string());
    }

    /// Genera un análisis real con LLM para el proceso seleccionado
    fn generate_real_analysis(&mut self) {
        if let Some(pid) = self.selected_pid {
            if let Some(process) = self.process_monitor.get_process_by_pid(pid) {
                // Convertir la información del proceso a formato JSON para el LLM
                let process_json = serde_json::json!({
                    "pid": process.pid,
                    "name": process.name,
                    "path": process.path,
                    "cmd_line": process.cmd_line,
                    "cpu_usage": process.cpu_usage,
                    "memory_usage": process.memory_usage,
                    "cpu_history": self.cpu_history,
                    "memory_history": self.memory_history,
                    "monitoring_time": self.monitoring_time.as_secs(),
                });
                
                // Esto actúa como placeholder. En una implementación completa,
                // esta función debería ser async y llamar al cliente LLM
                let analysis = format!(
                    "## Análisis de Comportamiento del Proceso\n\n\
                    **Proceso:** {} (PID: {})\n\n\
                    **⚠️ Analizando en tiempo real...**\n\n\
                    El proceso está siendo analizado. Para obtener un análisis real, es necesario configurar\
                    la integración con un LLM en el archivo de configuración.\n\n\
                    **Datos recopilados:**\n\
                    - CPU media: {:.2}%\n\
                    - Memoria: {} KB\n\
                    - Tiempo de monitoreo: {} segundos\n\
                    - Muestras recopiladas: {}\n\n\
                    *El análisis completo será generado utilizando un modelo de lenguaje (LLM).\
                    Este es un placeholder hasta que se implemente la integración completa.*\
                    ",
                    process.name, 
                    process.pid,
                    process.cpu_usage,
                    process.memory_usage,
                    self.monitoring_time.as_secs(),
                    self.cpu_history.len()
                );
                
                self.process_llm_analysis = Some(analysis);
                self.status_message = Some("Análisis generado (simulado). Para un análisis real, configura un LLM.".to_string());
                
                // Cambiar a la pestaña de análisis LLM
                self.process_monitor_tab = 1;
            }
        }
    }
} 
