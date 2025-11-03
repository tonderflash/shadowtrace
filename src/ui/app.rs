use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::ListState;
use ratatui::text::{Span, Line};
use ratatui::style::{Color, Style, Modifier};
use ratatui::widgets::{Block, Borders, Paragraph};
use serde_json;
use std::thread;
use std::sync::mpsc;

use crate::process::ProcessMonitor;
use crate::file_monitor::FileMonitor;
use crate::network::NetworkMonitor;
use crate::reports::Report;
use crate::llm::{LlmClient, LlmConfig, LlmProvider};

/// Estados posibles de la aplicación
pub enum AppState {
    Dashboard,
    ProcessMonitor,
    FileMonitor,
    NetworkMonitor,
    Reports,
    Help,
}

/// Información de sesión del proceso seleccionado (persiste entre cambios de pantalla)
#[derive(Clone)]
pub struct SelectedProcessSession {
    /// PID del proceso seleccionado
    pub pid: u32,
    /// Nombre del proceso cuando fue seleccionado
    pub name: String,
    /// Timestamp de cuando fue seleccionado
    pub selected_at: Instant,
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
    /// Sesión persistente del proceso seleccionado (mantiene estado entre pantallas)
    pub selected_process_session: Option<SelectedProcessSession>,
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
    /// Receptor para el resultado del análisis LLM (None si no hay análisis en curso)
    llm_analysis_rx: Option<mpsc::Receiver<Result<String, anyhow::Error>>>,
    /// Timestamp de cuando comenzó el análisis LLM
    llm_analysis_start_time: Option<Instant>,
    /// Animación del indicador de carga
    loading_tick: u64,
    /// Índice de desplazamiento para el texto LLM
    pub llm_text_scroll_index: Option<usize>,
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
            selected_process_session: None,
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
            llm_analysis_rx: None,
            llm_analysis_start_time: None,
            loading_tick: 0,
            llm_text_scroll_index: None,
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
        
        // Actualizar el indicador de carga si está activo
        self.update_loading_indicator();
        
        // Verificar timeout y estado del análisis LLM
        if let Some(start_time) = self.llm_analysis_start_time {
            let elapsed = self.last_tick.duration_since(start_time);
            let timeout_seconds = 120; // Timeout configurado en el cliente LLM
            
            // Verificar timeout
            if elapsed.as_secs() >= timeout_seconds {
                // Timeout alcanzado
                let error_msg = format!(
                    "Timeout: El análisis tardó más de {} segundos. Verifica la conexión al servicio LLM.",
                    timeout_seconds
                );
                self.status_message = Some(error_msg.clone());
                
                if let Some(pid) = self.selected_pid {
                    if let Some(process) = self.process_monitor.get_process_by_pid(pid) {
                        let timeout_analysis = format!(
                            "## ⚠️ Error de Timeout en el Análisis\n\n\
                            **Proceso:** {} (PID: {})\n\n\
                            **Estado:** El análisis ha excedido el tiempo máximo de espera ({} segundos).\n\n\
                            **Posibles causas:**\n\
                            - El servicio LLM no está respondiendo\n\
                            - Problemas de red o conectividad\n\
                            - El servicio está sobrecargado\n\n\
                            **Información del proceso:**\n\
                            - CPU: {:.2}%\n\
                            - Memoria: {} KB\n\
                            - Muestras recopiladas: {}\n\n\
                            **Recomendaciones:**\n\
                            1. Verifica tu conexión a internet\n\
                            2. Verifica que la API key de OpenAI sea válida\n\
                            3. Intenta nuevamente presionando 'A'\n\n\
                            *Tiempo transcurrido: {} segundos*\
                            ",
                            process.name,
                            process.pid,
                            timeout_seconds,
                            process.cpu_usage,
                            process.memory_usage,
                            self.cpu_history.len(),
                            elapsed.as_secs()
                        );
                        
                        self.process_llm_analysis = Some(timeout_analysis);
                    }
                }
                
                // Limpiar estado del análisis
                self.llm_analysis_rx = None;
                self.llm_analysis_start_time = None;
            } else {
                // Actualizar mensaje de estado con tiempo transcurrido
                let elapsed_secs = elapsed.as_secs();
                if elapsed_secs > 5 {
                    // Después de 5 segundos, mostrar tiempo transcurrido
                    self.status_message = Some(format!(
                        "Conectando con servicio LLM... ({}s / {}s timeout)",
                        elapsed_secs,
                        timeout_seconds
                    ));
                }
            }
        }
        
        // Verificar si hay resultados del análisis LLM
        if let Some(rx) = &self.llm_analysis_rx {
            if let Ok(analysis_result) = rx.try_recv() {
                // Recibimos un resultado, procesarlo
                let elapsed_time = self.llm_analysis_start_time
                    .map(|start| self.last_tick.duration_since(start).as_secs())
                    .unwrap_or(0);
                
                match analysis_result {
                    Ok(analysis) => {
                        // Actualizar el análisis y el estado
                        self.process_llm_analysis = Some(analysis);
                        self.status_message = Some(format!(
                            "Análisis completado con éxito en {} segundos",
                            elapsed_time
                        ));
                    },
                    Err(e) => {
                        // Mostrar un mensaje de error detallado
                        let error_msg = format!("Error al realizar análisis: {}", e);
                        self.status_message = Some(error_msg.clone());
                        
                        if let Some(pid) = self.selected_pid {
                            if let Some(process) = self.process_monitor.get_process_by_pid(pid) {
                                // Determinar tipo de error
                                let error_type = if error_msg.contains("timeout") || error_msg.contains("Timeout") {
                                    "Timeout de Conexión"
                                } else if error_msg.contains("connection") || error_msg.contains("Connection") {
                                    "Error de Conexión"
                                } else if error_msg.contains("404") || error_msg.contains("Not Found") {
                                    "Servicio No Encontrado"
                                } else if error_msg.contains("500") || error_msg.contains("Internal Server Error") {
                                    "Error del Servidor"
                                } else {
                                    "Error Desconocido"
                                };
                                
                                // Generar análisis alternativo con información detallada del error
                                let fallback_analysis = format!(
                                    "## ⚠️ Error al Conectar con el Servicio LLM\n\n\
                                    **Proceso:** {} (PID: {})\n\n\
                                    **Tipo de Error:** {}\n\n\
                                    **Detalles del Error:**\n\
                                    {}\n\n\
                                    **Tiempo transcurrido:** {} segundos\n\n\
                                    **Datos recopilados del proceso:**\n\
                                    - CPU promedio: {:.2}%\n\
                                    - Memoria: {} KB\n\
                                    - Tiempo de monitoreo: {} segundos\n\
                                    - Muestras recopiladas: {}\n\n\
                                    **Diagnóstico:**\n\
                                    - Servicio: OpenAI API (https://api.openai.com/v1/chat/completions)\n\
                                    - Modelo: gpt-4o-mini\n\
                                    - Timeout configurado: 120 segundos\n\n\
                                    **Soluciones sugeridas:**\n\
                                    1. Verifica tu conexión a internet\n\
                                    2. Comprueba que la API key de OpenAI sea válida y tenga créditos\n\
                                    3. Verifica que el endpoint sea accesible: `curl https://api.openai.com/v1/models`\n\
                                    4. Revisa tu cuenta de OpenAI para verificar el estado\n\
                                    5. Intenta nuevamente presionando 'A'\n\n\
                                    *Este análisis básico se generó debido al error de conexión.*\
                                    ",
                                    process.name, 
                                    process.pid,
                                    error_type,
                                    error_msg,
                                    elapsed_time,
                                    process.cpu_usage,
                                    process.memory_usage,
                                    self.monitoring_time.as_secs(),
                                    self.cpu_history.len()
                                );
                                
                                self.process_llm_analysis = Some(fallback_analysis);
                            }
                        }
                    }
                }
                
                // Limpiar estado del análisis
                self.llm_analysis_rx = None;
                self.llm_analysis_start_time = None;
            }
        }
        
        // Actualizar la lista de procesos cada 50 ticks (aproximadamente cada 5 segundos)
        if self.tick_count % 50 == 0 {
            self.refresh_processes();
        }
        
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
        
        // Sincronizar list_state con el proceso seleccionado persistente si existe
        if let Some(session) = &self.selected_process_session {
            if let Some(index) = self.processes.iter().position(|p| p.pid == session.pid) {
                self.list_state.select(Some(index));
                // Restaurar selected_pid si estaba desincronizado
                if self.selected_pid != Some(session.pid) {
                    self.selected_pid = Some(session.pid);
                }
            }
        }
        
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
            KeyCode::Char('p') => {
                self.state = AppState::ProcessMonitor;
                // Sincronizar la lista con el proceso seleccionado persistente
                self.sync_process_list_with_selection();
            },
            KeyCode::Char('f') => self.state = AppState::FileMonitor,
            KeyCode::Char('n') => self.state = AppState::NetworkMonitor,
            KeyCode::Char('r') => self.state = AppState::Reports,
            KeyCode::Char('h') => self.state = AppState::Help,
            _ => {}
        }
    }
    
    /// Sincroniza la lista de procesos con el proceso seleccionado persistente
    fn sync_process_list_with_selection(&mut self) {
        if let Some(session) = &self.selected_process_session {
            // Buscar el índice del proceso seleccionado en la lista actual
            if let Some(index) = self.processes.iter().position(|p| p.pid == session.pid) {
                self.list_state.select(Some(index));
                // Asegurar que selected_pid está sincronizado
                self.selected_pid = Some(session.pid);
            }
        }
    }
    
    /// Limpia todo el estado de la UI relacionado con el proceso anterior
    /// Esto asegura que no quede información "colgando" de procesos anteriores
    fn clear_process_state(&mut self) {
        // Detener monitoreo activo si existe
        if self.is_monitoring_active {
            self.is_monitoring_active = false;
        }
        
        // Limpiar análisis LLM
        self.process_llm_analysis = None;
        
        // Limpiar historiales de datos
        self.cpu_history.clear();
        self.memory_history.clear();
        
        // Resetear tiempos y duración de monitoreo
        self.monitoring_start_time = None;
        self.monitoring_time = Duration::from_secs(0);
        self.monitoring_duration = 0;
        
        // Limpiar receptor de análisis LLM si existe
        self.llm_analysis_rx = None;
        self.llm_analysis_start_time = None;
        
        // Resetear scroll del texto LLM
        self.llm_text_scroll_index = None;
        
        // Resetear a la pestaña de detalles (vista inicial limpia)
        self.process_monitor_tab = 0;
    }

    fn handle_process_monitor_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => self.state = AppState::Dashboard,
            KeyCode::Char('r') => self.refresh_processes(),
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Generar análisis real del proceso seleccionado
                if let Some(pid) = self.selected_pid {
                    // Verificar si hay datos de monitoreo suficientes
                    if self.cpu_history.len() < 5 && !self.is_monitoring_active {
                        self.status_message = Some("Se recomienda monitorear primero (tecla 'M') para mejores resultados".to_string());
                    } else {
                        // Cambiar a la pestaña de análisis LLM automáticamente
                        self.process_monitor_tab = 1;
                        self.generate_real_analysis();
                    }
                } else {
                    self.status_message = Some("Selecciona un proceso primero".to_string());
                }
            },
            KeyCode::Char('m') | KeyCode::Char('M') => {
                // Iniciar monitoreo si hay un proceso seleccionado
                if let Some(_) = self.selected_pid {
                    if !self.is_monitoring_active {
                        // Monitoreo por 30 segundos por defecto 
                        self.start_monitoring(30);
                    } else {
                        self.status_message = Some("Ya hay un monitoreo activo. Presiona 's' para detenerlo.".to_string());
                    }
                } else {
                    self.status_message = Some("Selecciona un proceso primero".to_string());
                }
            },
            KeyCode::Char('s') | KeyCode::Char('S') => {
                // Detener monitoreo activo
                if self.is_monitoring_active {
                    self.stop_monitoring();
                    // Sugerir análisis después de detener monitoreo
                    self.status_message = Some("Monitoreo detenido. Presiona 'a' para analizar los datos recopilados.".to_string());
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
                        let process = &self.processes[i];
                        let pid = process.pid;
                        let name = process.name.clone();
                        
                        // Verificar si se está seleccionando un proceso diferente
                        let is_different_process = self.selected_pid != Some(pid);
                        
                        // Si es un proceso diferente, limpiar todo el estado anterior
                        if is_different_process {
                            self.clear_process_state();
                        }
                        
                        // Establecer PID seleccionado
                        self.selected_pid = Some(pid);
                        
                        // Crear/actualizar sesión persistente del proceso seleccionado
                        self.selected_process_session = Some(SelectedProcessSession {
                            pid,
                            name,
                            selected_at: Instant::now(),
                        });
                        
                        self.status_message = Some(format!(
                            "Proceso seleccionado: {} (PID {}). Presiona 'm' para iniciar monitoreo o 'a' para análisis.", 
                            self.selected_process_session.as_ref().unwrap().name, pid
                        ));
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
        
        // Generar mensaje de estado basado en la cantidad de datos recopilados
        if self.cpu_history.len() >= 5 {
            self.status_message = Some(format!(
                "Monitoreo detenido. Se recopilaron {} muestras. Presiona 'A' para analizar.", 
                self.cpu_history.len()
            ));
            
            // Cambiar a la pestaña de análisis para guiar al usuario
            if self.process_monitor_tab == 0 {
                self.process_monitor_tab = 1;
            }
        } else if !self.cpu_history.is_empty() {
            self.status_message = Some(format!(
                "Monitoreo detenido. Solo se recopilaron {} muestras. Considera monitorear por más tiempo.", 
                self.cpu_history.len()
            ));
        } else {
            self.status_message = Some("Monitoreo detenido sin recopilar datos.".to_string());
        }
    }

    /// Genera un análisis real con LLM para el proceso seleccionado
    fn generate_real_analysis(&mut self) {
        if let Some(pid) = self.selected_pid {
            if let Some(process) = self.process_monitor.get_process_by_pid(pid) {
                // Si hay monitoreo activo, primero lo detenemos
                if self.is_monitoring_active {
                    self.stop_monitoring();
                    self.status_message = Some("Monitoreo detenido. Preparando análisis...".to_string());
                }

                // Actualizar mensaje de estado
                self.status_message = Some("Conectando con servicio LLM...".to_string());
                
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
                
                // Convertir eventos de archivo y red a formato JSON
                let file_events = self.file_monitor.get_events_for_pid(pid);
                let network_events = self.network_monitor.get_events_for_pid(pid);
                
                let file_events_json = serde_json::to_value(&file_events).unwrap_or_else(|_| serde_json::json!([]));
                let network_events_json = serde_json::to_value(&network_events).unwrap_or_else(|_| serde_json::json!([]));
                
                // Crear un reporte para este análisis
                let mut report = crate::reports::Report::new_for_process(pid, process.name.clone());
                report.set_process_info(process.clone());
                
                // Añadir datos de monitoreo al reporte
                if !self.cpu_history.is_empty() {
                    let avg_cpu = self.cpu_history.iter().sum::<f32>() / self.cpu_history.len() as f32;
                    let max_cpu = self.cpu_history.iter().fold(0.0f32, |max, &val| if val > max { val } else { max });
                    
                    report.add_info(
                        "monitoring", 
                        &format!("Datos de monitoreo UI: CPU promedio {:.2}%, máxima {:.2}%, tiempo {} segundos", 
                            avg_cpu, max_cpu, self.monitoring_time.as_secs()),
                        None
                    );
                }
                
                // Mostrar un análisis en estado de carga con indicador animado
                self.process_monitor_tab = 1; // Cambiar a la pestaña de análisis
                
                // Crear una cadena de texto con indicador de carga animado
                let process_name = process.name.clone();
                let process_pid = process.pid;
                let process_cpu = process.cpu_usage;
                let process_mem = process.memory_usage;
                let monitoring_time = self.monitoring_time.as_secs();
                let samples_count = self.cpu_history.len();
                
                // Mostrar análisis en estado de carga inicial
                let loading_analysis = format!(
                    "## Analizando Comportamiento del Proceso\n\n\
                    **Proceso:** {} (PID: {})\n\n\
                    **⏳ Conectando con el servicio de análisis...**\n\n\
                    Estado: Iniciando conexión al servicio LLM\n\
                    Tiempo transcurrido: 0s / 120s timeout\n\n\
                    Por favor espera mientras se procesa la información del proceso.\n\
                    Este análisis puede tardar unos segundos en completarse.\n\n\
                    **Datos que se están analizando:**\n\
                    - CPU media: {:.2}%\n\
                    - Memoria: {} KB\n\
                    - Tiempo de monitoreo: {} segundos\n\
                    - Muestras recopiladas: {}\n\n\
                    **Configuración del servicio:**\n\
                    - Endpoint: https://api.openai.com/v1/chat/completions\n\
                    - Modelo: gpt-4o-mini\n\
                    - Timeout: 120 segundos\n\n\
                    *La interfaz seguirá respondiendo mientras se realiza el análisis. \
                    El tiempo transcurrido se actualizará automáticamente.*\
                    ",
                    process_name.clone(), 
                    process_pid,
                    process_cpu,
                    process_mem,
                    monitoring_time,
                    samples_count
                );
                
                self.process_llm_analysis = Some(loading_analysis);
                
                // Configurar cliente LLM para usar OpenAI con modelo económico y eficiente
                // Obtener API key desde variable de entorno (según documentación de Rust)
                // std::env::var() devuelve Result<String, VarError>, usamos .ok() para Option
                let openai_key_result = std::env::var("OPENAI_API_KEY");
                let openai_key_alt_result = std::env::var("OPENAI_KEY");
                
                // Verificar qué variables están disponibles para el mensaje de error
                let openai_key_found = openai_key_result.is_ok();
                let openai_key_alt_found = openai_key_alt_result.is_ok();
                
                // Obtener la API key de la primera variable disponible
                let api_key = openai_key_result.ok().or_else(|| openai_key_alt_result.ok());
                
                // Validar que la API key esté configurada
                if api_key.is_none() {
                    // Debug: mostrar qué variables se están buscando
                    let env_check = format!(
                        "DEBUG: Se buscaron las siguientes variables de entorno:\n\
                        - OPENAI_API_KEY: {}\n\
                        - OPENAI_KEY: {}",
                        if openai_key_found { "✓ Encontrada" } else { "✗ No encontrada" },
                        if openai_key_alt_found { "✓ Encontrada" } else { "✗ No encontrada" }
                    );
                    
                    let error_msg = format!(
                        "❌ **Error: API Key de OpenAI no configurada**\n\n\
                        Para usar el análisis con OpenAI, necesitas configurar la variable de entorno.\n\n\
                        **Configuración rápida:**\n\
                        ```bash\n\
                        export OPENAI_API_KEY=\"tu-api-key-aqui\"\n\
                        ```\n\n\
                        **O desde tu archivo de configuración del shell (~/.zshrc, ~/.bashrc, etc.):**\n\
                        ```bash\n\
                        echo 'export OPENAI_API_KEY=\"tu-api-key-aqui\"' >> ~/.zshrc\n\
                        source ~/.zshrc\n\
                        ```\n\n\
                        También puedes usar `OPENAI_KEY` como alternativa.\n\n\
                        **Nota importante:**\n\
                        - La variable debe estar configurada en el mismo shell donde ejecutas ShadowTrace\n\
                        - Si ya la configuraste, reinicia la aplicación\n\
                        - Verifica con: `echo $OPENAI_API_KEY`\n\n\
                        **Instrucciones completas:**\n\
                        1. Obtén tu API key desde https://platform.openai.com/api-keys\n\
                        2. Configura la variable de entorno antes de ejecutar ShadowTrace\n\
                        3. Reinicia la aplicación después de configurar la variable\n\n\
                        ---\n\
                        {}\
                        ",
                        env_check
                    );
                    self.process_llm_analysis = Some(error_msg);
                    self.status_message = Some("Error: API Key de OpenAI no configurada".to_string());
                    return;
                }
                
                let llm_config = LlmConfig {
                    provider: LlmProvider::OpenAiCompatible,
                    api_url: "https://api.openai.com/v1/chat/completions".to_string(),
                    model: "gpt-4o-mini".to_string(), // Modelo económico pero potente
                    api_key: api_key.clone(),
                    temperature: 0.7,
                    timeout_seconds: 120,
                    max_tokens: Some(4096),
                };
                
                // Crear canal para recibir el resultado del análisis
                let (tx, rx) = mpsc::channel();
                
                // Guardar el receptor y timestamp en la estructura para procesarlo en tick()
                self.llm_analysis_rx = Some(rx);
                self.llm_analysis_start_time = Some(Instant::now());
                
                // Clonar los datos necesarios para el hilo
                let process_json_clone = process_json.clone();
                let file_events_json_clone = file_events_json.clone();
                let network_events_json_clone = network_events_json.clone();
                let llm_config_clone = llm_config.clone();
                let process_name_clone = process_name.clone();
                
                // Crear instrucciones de análisis personalizadas
                let custom_instructions = r#"
                Eres un modelo de lenguaje. No necesitas capacidades de ejecución para hacer análisis útil.

                A continuación te paso un reporte generado por un sistema llamado ShadowTrace. El reporte contiene texto estructurado con información de procesos, archivos accedidos y conexiones de red. No necesitas analizar archivos ni ejecutar código, solo interpretar lo que ya está escrito.

                Tu tarea es:
                    • Leer el reporte y explicar en lenguaje humano técnico qué está haciendo el proceso monitoreado.
                    • Comentar si el comportamiento es típico o sospechoso.
                    • Hacer hipótesis razonables sobre el propósito del proceso basándote en los datos observados (ej. si parece un navegador, si hay persistencia, si hay tráfico de red anómalo, etc.).
                    • Puedes mencionar si algo parece malicioso, sospechoso o simplemente automatizado.
                    • Tu análisis puede ser especulativo, pero bien fundamentado.
                    • NO digas que no puedes hacer el análisis. NO digas que eres solo un modelo de lenguaje. Ya lo sabemos. Estás aquí para interpretar texto. Hazlo como si fueras un analista de seguridad con experiencia.
                "#;
                
                // Lanzar hilo para realizar el análisis sin bloquear
                thread::spawn(move || {
                    // Esta parte se ejecuta en un hilo separado, por lo que es seguro crear un nuevo runtime aquí
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    let result = rt.block_on(async {
                        // Crear cliente LLM
                        match LlmClient::new(llm_config_clone) {
                            Ok(client) => {
                                // Añadir instrucciones personalizadas
                                let mut enriched_data = process_json_clone.clone();
                                enriched_data["instructions"] = serde_json::json!(custom_instructions);
                                
                                // Realizar análisis
                                let analysis_result = client.comprehensive_analysis(
                                    enriched_data,
                                    file_events_json_clone,
                                    network_events_json_clone
                                ).await;
                                
                                analysis_result
                            },
                            Err(e) => {
                                Err(anyhow::anyhow!("Error al crear cliente LLM: {}", e))
                            }
                        }
                    });
                    
                    // Enviar resultado al hilo principal a través del canal
                    let _ = tx.send(result);
                });
                
                // Actualizar estado pero no intentar procesar la respuesta aquí
                self.status_message = Some("Análisis en curso. Por favor espera...".to_string());
                
                // El resultado será procesado en el método tick()
            }
        }
    }

    // Añadir método para actualizar el indicador de carga
    fn update_loading_indicator(&mut self) {
        if let Some(analysis) = &mut self.process_llm_analysis {
            if analysis.contains("Conectando con el servicio de análisis...") {
                // Obtener tiempo transcurrido si hay un análisis en curso
                if let Some(start_time) = self.llm_analysis_start_time {
                    let elapsed = self.last_tick.duration_since(start_time);
                    let elapsed_secs = elapsed.as_secs();
                    let timeout_seconds = 120;
                    
                    // Actualizar el indicador de carga basado en el tick_count
                    let loading_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                    let idx = (self.tick_count / 5) % loading_chars.len() as u64;
                    let loading_char = loading_chars[idx as usize];
                    
                    // Determinar estado de conexión basado en el tiempo
                    let connection_status = if elapsed_secs < 3 {
                        "Iniciando conexión al servicio LLM"
                    } else if elapsed_secs < 10 {
                        "Estableciendo conexión..."
                    } else if elapsed_secs < 30 {
                        "Conectado, procesando solicitud..."
                    } else if elapsed_secs < 60 {
                        "Procesando análisis (esto puede tardar)..."
                    } else {
                        "Análisis en curso (tiempo prolongado)..."
                    };
                    
                    // Crear nuevo texto con tiempo actualizado
                    let new_analysis = format!(
                        "## Analizando Comportamiento del Proceso\n\n\
                        **Proceso:** {}\n\n\
                        **{} Conectando con el servicio de análisis...**\n\n\
                        **Estado:** {}\n\
                        **Tiempo transcurrido:** {}s / {}s timeout\n\n\
                        Por favor espera mientras se procesa la información del proceso.\n\
                        Este análisis puede tardar unos segundos en completarse.\n\n\
                        **Datos que se están analizando:**\n\
                        - CPU media: {:.2}%\n\
                        - Memoria: {} KB\n\
                        - Tiempo de monitoreo: {} segundos\n\
                        - Muestras recopiladas: {}\n\n\
                        **Configuración del servicio:**\n\
                        - Endpoint: https://api.openai.com/v1/chat/completions\n\
                        - Modelo: gpt-4o-mini\n\
                        - Timeout: {} segundos\n\n\
                        *La interfaz seguirá respondiendo mientras se realiza el análisis. \
                        El tiempo transcurrido se actualiza automáticamente.*\
                        ",
                        // Extraer nombre del proceso del análisis original
                        if let Some(pid) = self.selected_pid {
                            if let Some(process) = self.process_monitor.get_process_by_pid(pid) {
                                format!("{} (PID: {})", process.name, pid)
                            } else {
                                format!("PID: {}", pid)
                            }
                        } else {
                            "Desconocido".to_string()
                        },
                        loading_char,
                        connection_status,
                        elapsed_secs,
                        timeout_seconds,
                        // Estos valores necesitan ser extraídos del análisis original o recalculados
                        if let Some(pid) = self.selected_pid {
                            if let Some(process) = self.process_monitor.get_process_by_pid(pid) {
                                process.cpu_usage
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        },
                        if let Some(pid) = self.selected_pid {
                            if let Some(process) = self.process_monitor.get_process_by_pid(pid) {
                                process.memory_usage
                            } else {
                                0
                            }
                        } else {
                            0
                        },
                        self.monitoring_time.as_secs(),
                        self.cpu_history.len(),
                        timeout_seconds
                    );
                    
                    *analysis = new_analysis;
                }
            }
        }
    }

    /// Maneja el desplazamiento del texto de análisis LLM
    pub fn handle_llm_text_scroll(&mut self, key: KeyCode) {
        if let Some(analysis) = &self.process_llm_analysis {
            // Convertir el texto a líneas para calcular el tamaño total
            let total_lines = analysis.lines().count();
            
            // Obtener el índice actual de scroll o inicializarlo
            let current_index = self.llm_text_scroll_index.unwrap_or(0);
            
            // Calcular el nuevo índice según la tecla presionada
            let new_index = match key {
                KeyCode::Up => current_index.saturating_sub(1),
                KeyCode::Down => if current_index + 1 < total_lines {
                    current_index + 1
                } else {
                    current_index
                },
                KeyCode::PageUp => current_index.saturating_sub(10),
                KeyCode::PageDown => if current_index + 10 < total_lines {
                    current_index + 10
                } else {
                    total_lines.saturating_sub(1)
                },
                KeyCode::Home => 0,
                KeyCode::End => total_lines.saturating_sub(1),
                _ => current_index,
            };
            
            // Actualizar el índice de scroll
            self.llm_text_scroll_index = Some(new_index);
        }
    }
} 
