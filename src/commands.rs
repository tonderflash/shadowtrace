use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time;
use chrono::Utc;

use crate::process::ProcessMonitor;
use crate::file_monitor::{FileEvent, FileMonitor, FileOperation};
use crate::network::{NetworkEvent, NetworkMonitor, Protocol, Direction, ConnectionState};
use crate::reports::Report;
use crate::config::AppConfig;
use crate::error::AppError;

/// Monitorear un proceso específico
pub async fn monitor_process(
    pid: &Option<u32>,
    name: &Option<String>,
    duration: u64,
    interval_secs: u64,
    config: &AppConfig,
) -> Result<()> {
    // Inicializar monitores
    let mut process_monitor = ProcessMonitor::new();
    let mut file_monitor = FileMonitor::new();
    let mut network_monitor = NetworkMonitor::new();

    // Identificar el proceso
    let target_pid = match (pid, name) {
        (Some(p), _) => *p,
        (_, Some(n)) => {
            // Buscar proceso por nombre
            let processes = process_monitor.find_process_by_name(n);
            if processes.is_empty() {
                return Err(AppError::ProcessAccessError(format!("No se encontró ningún proceso con el nombre: {}", n)).into());
            } else if processes.len() > 1 {
                println!("Se encontraron múltiples procesos con el nombre '{}'. Seleccionando el primero.", n);
                for p in &processes {
                    println!("  PID: {}, CMD: {:?}", p.pid, p.cmd_line);
                }
            }
            processes[0].pid
        },
        _ => return Err(AppError::ConfigurationError("Debe especificar un PID o nombre de proceso".to_string()).into())
    };

    // Obtener información del proceso
    let process_info = process_monitor.get_process_by_pid(target_pid)
        .ok_or_else(|| AppError::ProcessAccessError(format!("No se encontró el proceso con PID: {}", target_pid)))?;

    // Iniciar reporte
    let mut report = Report::new(target_pid, process_info.name.clone());
    report.set_process_info(process_info.clone());
    
    // Mensaje de inicio
    println!("Monitoreando proceso: {} (PID: {})", process_info.name, target_pid);
    if let Some(path) = &process_info.path {
        println!("Ruta del ejecutable: {}", path);
    }
    if let Some(cmd) = &process_info.cmd_line {
        println!("Línea de comandos: {}", cmd.join(" "));
    }
    
    report.add_info(
        "monitor", 
        &format!("Iniciando monitoreo del proceso {} (PID: {})", 
            process_info.name, target_pid), 
        None
    );

    // Configurar loop de monitoreo
    let interval_duration = Duration::from_secs(interval_secs);
    let mut tick_interval = time::interval(interval_duration);
    
    // Tiempo máximo si no es infinito
    let max_iterations = if duration > 0 { 
        if interval_secs > 0 {
            Some(duration / interval_secs)
        } else {
            Some(duration) // Si el intervalo es 0, usamos la duración como número máximo de iteraciones
        }
    } else { 
        None // Si la duración es 0, monitoreamos indefinidamente
    };
    let mut iterations = 0;

    // Loop de monitoreo
    loop {
        tick_interval.tick().await;
        
        // Incrementar contador de iteraciones
        iterations += 1;
        
        // Verificar si debemos terminar
        if let Some(max) = max_iterations {
            if iterations >= max {
                break;
            }
        }
        
        // Actualizar información del proceso
        if let Some(updated_info) = process_monitor.get_process_by_pid(target_pid) {
            // Verificar si todavía está en ejecución
            if updated_info.cpu_usage == 0.0 && iterations > 2 {
                report.add_warning(
                    "process", 
                    &format!("El proceso {} (PID: {}) parece haber terminado", 
                        updated_info.name, target_pid), 
                    None
                );
                println!("⚠️ El proceso parece haber terminado (uso de CPU: 0%)");
                break;
            }
            
            // Registrar uso de recursos
            let cpu_usage = updated_info.cpu_usage;
            let memory_usage = updated_info.memory_usage;
            
            if cpu_usage > 80.0 {
                report.add_warning(
                    "resource", 
                    &format!("Alto uso de CPU: {:.2}%", cpu_usage), 
                    None
                );
            }
            
            if iterations % 5 == 0 {
                println!("Uso CPU: {:.2}%, Memoria: {} KB", cpu_usage, memory_usage);
            }
        } else {
            report.add_warning("process", "Proceso terminado o no accesible", None);
            println!("⚠️ El proceso ya no está accesible");
            break;
        }
        
        // Simular eventos de archivo y red (aquí iría la implementación real)
        simulate_file_events(&mut file_monitor, &mut report, target_pid, iterations);
        simulate_network_events(&mut network_monitor, &mut report, target_pid, iterations);
        
        // Detectar patrones sospechosos
        detect_file_patterns(&file_monitor, &mut report, target_pid);
        detect_network_patterns(&network_monitor, &mut report, target_pid);
    }
    
    // Finalizar monitoreo
    report.update_end_time();
    println!("Monitoreo finalizado para {} (PID: {})", process_info.name, target_pid);
    
    // Analizar con LLM si está disponible
    if let Some(client) = &config.llm_client {
        println!("Analizando comportamiento con IA...");
        
        // Convertir a JSON para el LLM
        let process_json = serde_json::to_value(&process_info)?;
        let file_events_json = serde_json::to_value(&file_monitor.get_events_for_pid(target_pid))?;
        let network_events_json = serde_json::to_value(&network_monitor.get_events_for_pid(target_pid))?;
        
        // Realizar análisis completo
        match client.comprehensive_analysis(
            process_json,
            file_events_json,
            network_events_json,
        ).await {
            Ok(analysis) => {
                report.set_llm_analysis(analysis.clone());
                println!("\n--- Análisis de IA ---\n{}\n", analysis);
            }
            Err(e) => {
                println!("⚠️ Error al realizar análisis con LLM: {}. Continuando sin análisis.", e);
            }
        }
    }
    
    // Guardar reportes
    match report.save_to_default_dir() {
        Ok((json_path, md_path)) => {
            println!("Reporte JSON guardado en: {}", json_path.display());
            println!("Reporte Markdown guardado en: {}", md_path.display());
        }
        Err(e) => {
            println!("⚠️ Error al guardar reportes: {}. Continuando sin guardar reportes.", e);
        }
    }
    
    Ok(())
}

/// Simular eventos de archivo
fn simulate_file_events(
    file_monitor: &mut FileMonitor, 
    report: &mut Report, 
    target_pid: u32, 
    iterations: u64
) {
    if iterations % 3 == 0 {
        // Usar rutas compatibles con el sistema operativo
        #[cfg(target_os = "linux")]
        let file_path = format!("/tmp/test_file_{}.txt", iterations);
        
        #[cfg(target_os = "macos")]
        let file_path = format!("/tmp/test_file_{}.txt", iterations);
        
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        let file_path = format!("C:/temp/test_file_{}.txt", iterations);
        
        let event = FileEvent {
            pid: target_pid,
            path: file_path,
            operation: FileOperation::Write,
            timestamp: Utc::now(),
            size: Some(1024),
            success: true,
        };
        file_monitor.record_event(event.clone());
        report.add_file_event(event);
    }
}

/// Simular eventos de red
fn simulate_network_events(
    network_monitor: &mut NetworkMonitor, 
    report: &mut Report, 
    target_pid: u32, 
    iterations: u64
) {
    if iterations % 4 == 0 {
        let event = NetworkEvent {
            pid: target_pid,
            local_addr: "127.0.0.1:12345".parse().unwrap(),
            remote_addr: Some("8.8.8.8:443".parse().unwrap()),
            protocol: Protocol::TCP,
            direction: Direction::Outbound,
            state: ConnectionState::Established,
            timestamp: Utc::now(),
            bytes_sent: Some(512),
            bytes_received: Some(1024),
        };
        network_monitor.record_event(event.clone());
        report.add_network_event(event);
    }
}

/// Detectar patrones sospechosos de archivos
fn detect_file_patterns(
    file_monitor: &FileMonitor, 
    report: &mut Report, 
    target_pid: u32
) {
    let suspicious_files = file_monitor.detect_suspicious_patterns(target_pid);
    for pattern in suspicious_files {
        report.add_alert("file_access", &pattern, None);
        println!("⚠️ {}", pattern);
    }
}

/// Detectar patrones sospechosos de red
fn detect_network_patterns(
    network_monitor: &NetworkMonitor, 
    report: &mut Report, 
    target_pid: u32
) {
    let suspicious_network = network_monitor.detect_suspicious_patterns(target_pid);
    for pattern in suspicious_network {
        report.add_alert("network", &pattern, None);
        println!("⚠️ {}", pattern);
    }
}

/// Auditar un binario
pub async fn audit_binary(
    binary: &PathBuf,
    _args: &Option<Vec<String>>,
    _timeout: u64,
    _config: &AppConfig,
) -> Result<()> {
    println!("Auditando binario: {:?}", binary);
    println!("Función no implementada completamente");
    
    // Aquí iría el código para ejecutar el binario en un entorno controlado
    // y monitorear su comportamiento
    
    Ok(())
}

/// Monitorear actividad del sistema
pub async fn monitor_system(
    watch: bool,
    duration: u64,
    suspicious_only: bool,
    _config: &AppConfig,
) -> Result<()> {
    println!("Monitoreando sistema: watch={}, duration={}, suspicious_only={}", 
        watch, duration, suspicious_only);
    println!("Función no implementada completamente");
    
    // Aquí iría el código para monitorear la actividad del sistema
    
    Ok(())
} 
