use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::App;

pub fn draw_help(frame: &mut Frame, _app: &mut App) {
    let size = frame.area();
    
    // Dividir la pantalla en secciones
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Título
            Constraint::Min(10),    // Contenido
            Constraint::Length(3),  // Barra de estado
        ].as_ref())
        .split(size);
    
    // Título
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Ayuda de ShadowTrace", 
            Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))
    ]))
    .alignment(ratatui::layout::Alignment::Center)
    .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Blue)));
    
    frame.render_widget(title, chunks[0]);
    
    // Contenido de ayuda
    let help_text = vec![
        Line::from(vec![
            Span::styled("🔍 Conceptos Básicos", 
                Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![
            Span::raw("ShadowTrace es una herramienta de monitoreo de procesos con capacidades de análisis avanzado.")
        ]),
        Line::from(vec![
            Span::raw("La aplicación permite observar en tiempo real el comportamiento de procesos, incluyendo:")
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::LightGreen)),
            Span::raw("Uso de CPU y memoria")
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::LightGreen)),
            Span::raw("Operaciones de archivo")
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::LightGreen)),
            Span::raw("Actividad de red")
        ]),
        Line::from(vec![
            Span::styled("  • ", Style::default().fg(Color::LightGreen)),
            Span::raw("Análisis de patrones sospechosos")
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![
            Span::styled("⌨️ Atajos de Teclado", 
                Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::LightCyan)),
            Span::raw(" - Volver al menú principal / salir de la pantalla actual")
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓", Style::default().fg(Color::LightCyan)),
            Span::raw(" - Navegar por las listas")
        ]),
        Line::from(vec![
            Span::styled("  ENTER", Style::default().fg(Color::LightCyan)),
            Span::raw(" - Seleccionar opción")
        ]),
        Line::from(vec![
            Span::styled("  P", Style::default().fg(Color::LightCyan)),
            Span::raw(" - Ir a monitoreo de procesos")
        ]),
        Line::from(vec![
            Span::styled("  F", Style::default().fg(Color::LightCyan)),
            Span::raw(" - Ir a monitoreo de archivos")
        ]),
        Line::from(vec![
            Span::styled("  N", Style::default().fg(Color::LightCyan)),
            Span::raw(" - Ir a monitoreo de red")
        ]),
        Line::from(vec![
            Span::styled("  R", Style::default().fg(Color::LightCyan)),
            Span::raw(" - Ir a reportes")
        ]),
        Line::from(vec![
            Span::styled("  H", Style::default().fg(Color::LightCyan)),
            Span::raw(" - Mostrar esta ayuda")
        ]),
        Line::from(vec![
            Span::styled("  Q", Style::default().fg(Color::LightCyan)),
            Span::raw(" - Salir de la aplicación")
        ]),
    ];
    
    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Instrucciones "))
        .style(Style::default())
        .wrap(Wrap { trim: true });
    
    frame.render_widget(help_paragraph, chunks[1]);
    
    // Barra de estado
    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(" ℹ️ ", Style::default().fg(Color::LightYellow)),
        Span::raw("Presiona ESC o Q para volver al menú principal"),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default());
    
    frame.render_widget(status_bar, chunks[2]);
} 
