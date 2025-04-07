use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event as CEvent, KeyEvent, KeyEventKind};
use anyhow::Result;

pub enum Event<I> {
    Input(I),
    Tick,
}

/// Un manejador de eventos para la interfaz de usuario
pub struct Events {
    rx: mpsc::Receiver<Event<CEvent>>,
    _tx: mpsc::Sender<Event<CEvent>>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(tick_rate.as_millis() as u64);

        let event_tx = tx.clone();
        let tick_tx = tx.clone();
        
        // Hilo dedicado exclusivamente para eventos de teclado
        thread::spawn(move || {
            loop {
                // Usar read() bloqueante para capturar todos los eventos con certeza
                match event::read() {
                    Ok(event) => {
                        if let Err(err) = event_tx.send(Event::Input(event)) {
                            eprintln!("Error enviando evento: {:?}", err);
                            break;
                        }
                    },
                    Err(err) => {
                        eprintln!("Error leyendo evento: {:?}", err);
                    }
                }
            }
        });
        
        // Hilo para ticks con frecuencia reducida
        thread::spawn(move || {
            let mut last_tick = std::time::Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                
                if timeout.as_secs() == 0 && timeout.subsec_nanos() == 0 {
                    if let Err(err) = tick_tx.send(Event::Tick) {
                        eprintln!("Error enviando tick: {:?}", err);
                        break;
                    }
                    last_tick = std::time::Instant::now();
                } else {
                    thread::sleep(Duration::from_millis(50));
                }
            }
        });
        
        Self { rx, _tx: tx }
    }
    
    pub fn next(&self) -> Result<Option<CEvent>> {
        match self.rx.recv_timeout(Duration::from_millis(50)) {
            Ok(Event::Input(event)) => Ok(Some(event)),
            Ok(Event::Tick) => Ok(None),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(anyhow::anyhow!("Canal de eventos desconectado")),
        }
    }
}
