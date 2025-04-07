use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use crate::ui::events::Events;
use crate::ui::{
    AnimatedText, AnimatedTextState, ScannerText, ScannerTextState, SparklineBraille,
    BrailleChart, Dataset, Axis, AnimationType
};

/// Estructura principal de la aplicación
pub struct App {
    /// Título de la aplicación
    title: String,
    /// ¿La aplicación debería salir?
    should_quit: bool,
    /// Contador de ticks
    tick_counter: u64,
    /// Último tick
    last_tick: Instant,
    /// Datos de ejemplo para el gráfico
    chart_data: Vec<(f64, f64)>,
    /// Datos de ejemplo para el sparkline
    sparkline_data: Vec<u64>,
    /// Estado del texto animado
    animated_text_state: AnimatedTextState,
    /// Estado del texto con escáner
    scanner_text_state: ScannerTextState,
}

impl Default for App {
    fn default() -> Self {
        // Generar datos de ejemplo para el gráfico
        let chart_data = (0..100)
            .map(|i| {
                let x = i as f64 * 0.1;
                let y = (x * 0.8).sin() * 5.0 + ((x * 0.2).cos() * 3.0) + ((x * 0.5).sin() * 2.0);
                (x, y)
            })
            .collect();

        // Generar datos de ejemplo para el sparkline
        let sparkline_data = (0..100)
            .map(|i| {
                let x = i as f64 * 0.1;
                let value = ((x * 0.8).sin() * 50.0 + 50.0) as u64;
                value
            })
            .collect();

        Self {
            title: "ShadowTrace".to_string(),
            should_quit: false,
            tick_counter: 0,
            last_tick: Instant::now(),
            chart_data,
            sparkline_data,
            animated_text_state: AnimatedTextState::default()
                .set_animation_type(AnimationType::Wave),
            scanner_text_state: ScannerTextState::default(),
        }
    }
}

impl App {
    /// Crear una nueva instancia de la aplicación
    pub fn new() -> Self {
        Self::default()
    }

    /// Manejar eventos de teclas
    pub fn handle_key(&mut self, key: char) {
        match key {
            'q' => self.should_quit = true,
            '1' => self.animated_text_state = AnimatedTextState::default()
                .set_animation_type(AnimationType::Wave),
            '2' => self.animated_text_state = AnimatedTextState::default()
                .set_animation_type(AnimationType::Pulse),
            '3' => self.animated_text_state = AnimatedTextState::default()
                .set_animation_type(AnimationType::Matrix),
            '4' => self.animated_text_state = AnimatedTextState::default()
                .set_animation_type(AnimationType::Spiral),
            '5' => self.animated_text_state = AnimatedTextState::default()
                .set_animation_type(AnimationType::Scanner),
            _ => {}
        }
    }

    /// Actualizar el estado de la aplicación
    pub fn tick(&mut self) {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        self.last_tick = Instant::now();

        // Actualizar datos de ejemplo - Simular datos en tiempo real
        let last_value = self.sparkline_data.last().unwrap_or(&50);
        let new_value = (*last_value as f64 + (rand::random::<f64>() * 10.0 - 5.0)) as u64;
        self.sparkline_data.push(new_value.max(1).min(100));
        if self.sparkline_data.len() > 100 {
            self.sparkline_data.remove(0);
        }
    }

    /// ¿La aplicación debe salir?
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Dibujar la interfaz
    pub fn draw(&mut self, f: &mut Frame) {
        let size = f.area();

        // Crear layout principal
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Título
                Constraint::Length(8),  // Texto animado
                Constraint::Length(5),  // Sparkline
                Constraint::Min(10),    // Gráfico
                Constraint::Length(3),  // Pie de página
            ])
            .split(size);

        // Renderizar título
        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title(format!(" {} ", self.title));

        let title_paragraph = Paragraph::new(format!(
            "ShadowTrace - ASCII UI Demo [Tick: {}]",
            self.tick_counter
        ))
        .block(title_block)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        f.render_widget(title_paragraph, chunks[0]);

        // Renderizar texto animado
        let animated_text = AnimatedText::new("Presiona 1-5 para cambiar el tipo de animación")
            .block(Block::default().borders(Borders::ALL).title(" Animación "))
            .style(Style::default().fg(Color::Cyan));

        f.render_stateful_widget(animated_text, chunks[1], &mut self.animated_text_state);

        // Renderizar sparkline
        let sparkline_data: Vec<f64> = self.sparkline_data.iter()
            .map(|&x| x as f64)
            .collect();
        
        let sparkline = SparklineBraille::new(&sparkline_data)
            .block(Block::default().borders(Borders::ALL).title(" Actividad "))
            .style(Style::default().fg(Color::Green));

        f.render_widget(sparkline, chunks[2]);

        // Renderizar gráfico
        let dataset = Dataset::new("Datos", self.chart_data.clone())
            .style(Style::default().fg(Color::Magenta));

        let chart = BrailleChart::new(vec![dataset])
            .block(Block::default().borders(Borders::ALL).title(" Gráfico "))
            .x_axis(
                Axis::default()
                    .title("Tiempo")
                    .bounds([0.0, 10.0])
                    .labels(vec![
                        Span::raw("0.0"),
                        Span::raw("2.5"),
                        Span::raw("5.0"),
                        Span::raw("7.5"),
                        Span::raw("10.0"),
                    ])
            )
            .y_axis(
                Axis::default()
                    .title("Valor")
                    .bounds([-10.0, 10.0])
                    .labels(vec![
                        Span::raw("-10"),
                        Span::raw("-5"),
                        Span::raw("0"),
                        Span::raw("5"),
                        Span::raw("10"),
                    ])
            );

        f.render_widget(chart, chunks[3]);

        // Renderizar pie de página
        let footer_text = ScannerText::new("q: Salir | 1-5: Cambiar animación")
            .block(Block::default().borders(Borders::ALL).title(" Ayuda "))
            .style(Style::default().fg(Color::Gray))
            .scanner_style(Style::default().fg(Color::White).bg(Color::Blue));

        f.render_stateful_widget(footer_text, chunks[4], &mut self.scanner_text_state);
    }
}

/// Ejecutar la aplicación
pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| app.draw(f))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    crossterm::event::KeyCode::Char(c) => app.handle_key(c),
                    crossterm::event::KeyCode::Esc => app.should_quit = true,
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }

        if app.should_quit() {
            return Ok(());
        }
    }
} 
