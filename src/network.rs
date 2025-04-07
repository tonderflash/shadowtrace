use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Tipo de protocolo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Protocol {
    /// TCP
    TCP,
    /// UDP
    UDP,
    /// ICMP
    ICMP,
    /// Otro o desconocido
    Other,
}

/// Dirección de la conexión
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    /// Entrante
    Inbound,
    /// Saliente
    Outbound,
}

/// Estado de la conexión
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionState {
    /// Establecida
    Established,
    /// Conectando
    Connecting,
    /// Escuchando
    Listening,
    /// Cerrando
    Closing,
    /// Cerrada
    Closed,
    /// Otro o desconocido
    Other,
}

/// Evento de conexión de red
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    /// ID del proceso
    pub pid: u32,
    /// Dirección local
    pub local_addr: SocketAddr,
    /// Dirección remota
    pub remote_addr: Option<SocketAddr>,
    /// Protocolo
    pub protocol: Protocol,
    /// Dirección (entrante/saliente)
    pub direction: Direction,
    /// Estado de la conexión
    pub state: ConnectionState,
    /// Momento del evento
    pub timestamp: DateTime<Utc>,
    /// Bytes enviados (si aplica)
    pub bytes_sent: Option<u64>,
    /// Bytes recibidos (si aplica)
    pub bytes_received: Option<u64>,
}

/// Monitor de red
pub struct NetworkMonitor {
    /// Historial de eventos
    events: Vec<NetworkEvent>,
    /// Conexiones activas por PID
    active_connections: HashMap<u32, Vec<NetworkEvent>>,
}

impl NetworkMonitor {
    /// Crear un nuevo monitor de red
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            active_connections: HashMap::new(),
        }
    }

    /// Registrar un evento de red
    pub fn record_event(&mut self, event: NetworkEvent) {
        // Actualizar las conexiones activas
        match event.state {
            ConnectionState::Established | ConnectionState::Connecting | ConnectionState::Listening => {
                self.active_connections
                    .entry(event.pid)
                    .or_insert_with(Vec::new)
                    .push(event.clone());
            }
            ConnectionState::Closed => {
                if let Some(connections) = self.active_connections.get_mut(&event.pid) {
                    connections.retain(|conn| {
                        conn.local_addr != event.local_addr || 
                        conn.remote_addr != event.remote_addr
                    });
                }
            }
            _ => {}
        }

        self.events.push(event);
    }

    /// Obtener todos los eventos registrados
    pub fn get_events(&self) -> &[NetworkEvent] {
        &self.events
    }

    /// Obtener eventos para un proceso específico
    pub fn get_events_for_pid(&self, pid: u32) -> Vec<&NetworkEvent> {
        self.events.iter().filter(|e| e.pid == pid).collect()
    }

    /// Obtener conexiones activas para un proceso
    pub fn get_active_connections_for_pid(&self, pid: u32) -> Vec<&NetworkEvent> {
        match self.active_connections.get(&pid) {
            Some(connections) => connections.iter().collect(),
            None => Vec::new(),
        }
    }

    /// Limpiar eventos antiguos
    pub fn clean_old_events(&mut self, keep_count: usize) {
        if self.events.len() > keep_count {
            let to_remove = self.events.len() - keep_count;
            self.events.drain(0..to_remove);
        }
    }

    /// Detectar patrones sospechosos de red
    pub fn detect_suspicious_patterns(&self, pid: u32) -> Vec<String> {
        let events = self.get_events_for_pid(pid);
        let mut suspicious = Vec::new();
        
        // Detector de muchas conexiones en poco tiempo
        let mut connection_count_by_minute: HashMap<i64, usize> = HashMap::new();
        
        for event in &events {
            if event.state == ConnectionState::Established {
                let minute = event.timestamp.timestamp() / 60;
                *connection_count_by_minute.entry(minute).or_insert(0) += 1;
            }
        }
        
        for (_minute, count) in connection_count_by_minute {
            if count > 10 {
                suspicious.push(format!("Alta tasa de conexiones: {} en un minuto", count));
            }
        }
        
        // Detector de puertos sensibles
        let sensitive_ports = [22, 23, 3389, 445, 135, 139];
        
        for event in &events {
            if let Some(addr) = event.remote_addr {
                for port in &sensitive_ports {
                    if addr.port() == *port {
                        suspicious.push(format!("Conexión a puerto sensible: {}", addr));
                    }
                }
            }
        }
        
        // Detector de IPs sospechosas
        // En una implementación real, se verificaría contra listas de IPs maliciosas
        
        suspicious
    }
} 
