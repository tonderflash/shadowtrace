use chrono::{DateTime, Utc, TimeZone};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::SystemTime;

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

/// Conexión de red
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// Protocolo usado
    pub protocol: Protocol,
    /// Dirección local
    pub local_addr: SocketAddr,
    /// Dirección remota
    pub remote_addr: SocketAddr,
    /// Estado de la conexión (para TCP)
    pub state: Option<String>,
    /// PID del proceso asociado
    pub pid: Option<u32>,
    /// Timestamp de la primera vez que se vio
    pub first_seen: SystemTime,
    /// Timestamp de la última vez que se vio
    pub last_seen: SystemTime,
    /// Bytes enviados
    pub bytes_sent: u64,
    /// Bytes recibidos
    pub bytes_received: u64,
}

/// Monitor de red
pub struct NetworkMonitor {
    /// Conexiones activas
    connections: Vec<Connection>,
    /// Historial de eventos
    events: Vec<NetworkEvent>,
    /// Filtrar por PID
    filter_pid: Option<u32>,
}

impl NetworkMonitor {
    /// Crear un nuevo monitor de red
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            events: Vec::new(),
            filter_pid: None,
        }
    }

    /// Establecer filtro por PID
    pub fn set_pid_filter(&mut self, pid: Option<u32>) {
        self.filter_pid = pid;
    }

    /// Obtener las conexiones activas
    pub fn get_connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Obtener los eventos registrados
    pub fn get_events(&self) -> &[NetworkEvent] {
        &self.events
    }
    
    /// Simular detección de actividad de red para pruebas
    pub fn simulate_activity(&mut self) {
        // Generar una conexión simulada
        let remote_ports = [80, 443, 8080, 22, 25, 53];
        let protocols = [Protocol::TCP, Protocol::UDP];
        
        let timestamp = SystemTime::now();
        let remote_port = remote_ports[self.events.len() % remote_ports.len()];
        let protocol = protocols[self.events.len() % protocols.len()];
        
        // Crear un evento simulado
        let event = NetworkEvent {
            timestamp: Utc.timestamp_opt(
                timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as i64,
                0
            ).unwrap(),
            protocol,
            direction: Direction::Outbound,
            local_addr: "127.0.0.1:12345".parse().unwrap(),
            remote_addr: Some(format!("93.184.216.34:{}", remote_port).parse().unwrap()),
            state: ConnectionState::Established,
            pid: self.filter_pid.unwrap_or(0),
            bytes_sent: Some((1024 * ((self.events.len() % 10) + 1)) as u64),
            bytes_received: Some((2048 * ((self.events.len() % 10) + 1)) as u64),
        };
        
        self.events.push(event);
        
        // Limitar el historial a 100 eventos
        if self.events.len() > 100 {
            self.events.remove(0);
        }
        
        // Actualizar o crear conexiones
        if self.connections.len() < 5 {
            // Crear nuevas conexiones simuladas
            let connection = Connection {
                protocol,
                local_addr: "127.0.0.1:12345".parse().unwrap(),
                remote_addr: format!("93.184.216.34:{}", remote_port).parse().unwrap(),
                state: Some("ESTABLISHED".to_string()),
                pid: self.filter_pid,
                first_seen: timestamp,
                last_seen: timestamp,
                bytes_sent: (1024 * ((self.connections.len() % 10) + 1)) as u64,
                bytes_received: (2048 * ((self.connections.len() % 10) + 1)) as u64,
            };
            
            self.connections.push(connection);
        } else {
            // Actualizar una conexión existente
            if let Some(conn) = self.connections.iter_mut().next() {
                conn.last_seen = timestamp;
                conn.bytes_sent += 512;
                conn.bytes_received += 1024;
            }
        }
    }

    /// Registrar un evento de red
    pub fn record_event(&mut self, event: NetworkEvent) {
        // Actualizar las conexiones activas
        match event.state {
            ConnectionState::Established | ConnectionState::Connecting | ConnectionState::Listening => {
                self.connections
                    .iter_mut()
                    .filter(|conn| conn.local_addr == event.local_addr && 
                           event.remote_addr.as_ref().map_or(false, |addr| &conn.remote_addr == addr))
                    .for_each(|conn| {
                        conn.last_seen = SystemTime::now();
                        conn.bytes_sent += event.bytes_sent.unwrap_or(0);
                        conn.bytes_received += event.bytes_received.unwrap_or(0);
                    });
            }
            ConnectionState::Closed => {
                self.connections.retain(|conn| {
                    conn.local_addr != event.local_addr || 
                    event.remote_addr.as_ref().map_or(true, |addr| conn.remote_addr != *addr)
                });
            }
            _ => {}
        }

        self.events.push(event);
    }

    /// Obtener eventos para un proceso específico
    pub fn get_events_for_pid(&self, pid: u32) -> Vec<&NetworkEvent> {
        self.events.iter().filter(|e| e.pid == pid).collect()
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
