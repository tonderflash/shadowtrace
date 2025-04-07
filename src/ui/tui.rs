use std::io;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal, Frame,
};

use super::App;
use super::screens;
use super::events::Events;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    events: Events,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        let events = Events::new(Duration::from_millis(250));
        
        Ok(Self { terminal, events })
    }

    pub fn init(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| {
            match app.state {
                super::app::AppState::Dashboard => screens::draw_dashboard(frame, app),
                super::app::AppState::ProcessMonitor => screens::draw_process_monitor(frame, app),
                super::app::AppState::FileMonitor => screens::draw_file_monitor(frame, app),
                super::app::AppState::NetworkMonitor => screens::draw_network_monitor(frame, app),
                super::app::AppState::Reports => screens::draw_reports(frame, app),
                super::app::AppState::Help => screens::draw_help(frame, app),
            }
        })?;
        
        Ok(())
    }

    pub fn handle_events(&mut self, app: &mut App) -> Result<()> {
        if let Some(event) = self.events.next()? {
            match event {
                Event::Key(key_event) => {
                    if let KeyCode::Char(c) = key_event.code {
                        app.status_message = Some(format!("Tecla presionada: {}", c));
                    } else {
                        app.status_message = Some(format!("Tecla especial presionada"));
                    }
                    
                    app.handle_key_event(key_event);
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    pub fn run(&mut self, app: &mut App) -> Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(100); // Reducir la tasa de refresco para priorizar eventos

        while app.running {
            // Dibujar la interfaz
            self.draw(app)?;
            
            // Manejar eventos con prioridad
            self.handle_events(app)?;

            // Actualizar estado según tick rate
            if last_tick.elapsed() >= tick_rate {
                app.tick();
                last_tick = Instant::now();
            }
            
            // Pequeña pausa para evitar alto uso de CPU
            std::thread::sleep(Duration::from_millis(10));
        }
        
        Ok(())
    }
} 
