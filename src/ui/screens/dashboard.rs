use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::App;
use crate::ui::braille_art::{BrailleAnimator, AnimationType};

pub fn draw_dashboard(frame: &mut Frame, app: &mut App) {
    let size = frame.area();
    
    // Dividir la pantalla en secciones
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Título y banner
            Constraint::Length(10), // Animación
            Constraint::Min(10),    // Menú principal
        ].as_ref())
        .split(size);
    
    // Banner título
    draw_title_banner(frame, app, chunks[0]);
    
    // Animación
    draw_animation(frame, app, chunks[1]);
    
    // Menú principal
    draw_main_menu(frame, app, chunks[2]);
}

fn draw_title_banner(frame: &mut Frame, _app: &mut App, area: Rect) {
    let title_text = vec![
        Line::from(vec![
            Span::styled("██████  █████  ██   ██  █████  ██████   ██████  ██     ██", 
                Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("██     ██   ██ ██   ██ ██   ██ ██   ██ ██    ██ ██     ██", 
                Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("███████ ██████ ███████ ██   ██ █████   ██    ██ ██  █  ██", 
                Style::default().fg(Color::LightCyan)),
        ]),
        Line::from(vec![
            Span::styled("     ██ ██   ██ ██   ██ ██   ██ ██  ██  ██    ██ ██ ███ ██", 
                Style::default().fg(Color::LightCyan)),
        ]),
        Line::from(vec![
            Span::styled("███████ ██   ██ ██   ██  █████  ██   ██  ██████   ███ ███ ", 
                Style::default().fg(Color::LightCyan)),
        ]),
        Line::from(vec![
            Span::styled("                          TRACE", 
                Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
        ]),
    ];
    
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center);
    
    frame.render_widget(title, area);
}

fn draw_animation(frame: &mut Frame, app: &mut App, area: Rect) {
    // Crear bloque para la animación
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Blue))
        .title(Span::styled(" Monitoreo en Tiempo Real ", Style::default().fg(Color::Yellow)));
    
    // Calcular área interna disponible para la animación
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    // Ancho braille es mitad del ancho de caracteres (2 píxeles por caracter)
    let braille_width = inner_area.width as usize * 2;
    // Alto braille es un cuarto del alto de caracteres (4 píxeles por caracter)
    let braille_height = inner_area.height as usize * 4;
    
    // Seleccionar tipo de animación basado en el estado
    let animation_type = match app.tick_count % 500 {
        t if t < 100 => AnimationType::Wave,
        t if t < 200 => AnimationType::Pulse, 
        t if t < 300 => AnimationType::Matrix,
        t if t < 400 => AnimationType::Spiral,
        _ => AnimationType::Scanner,
    };
    
    // Crear y actualizar animador
    let mut animator = BrailleAnimator::new(braille_width, braille_height, animation_type);
    animator.update(None);
    
    // Renderizar la animación como Paragraph
    let animation_text = animator.render();
    let animation_paragraph = Paragraph::new(animation_text);
    
    frame.render_widget(animation_paragraph, inner_area);
}

fn draw_main_menu(frame: &mut Frame, _app: &mut App, area: Rect) {
    // Dividir el área en columnas
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ].as_ref())
        .split(area);
    
    // Opciones de menú
    let menu_items = [
        (
            " 📊 Monitoreo de Procesos (P) ",
            "Monitorea en tiempo real procesos del sistema con análisis detallado de comportamiento",
            Color::Green
        ),
        (
            " 📁 Monitoreo de Archivos (F) ",
            "Observa operaciones de archivos realizadas por los procesos monitoreados",
            Color::Yellow
        ),
        (
            " 🌐 Monitoreo de Red (N) ",
            "Visualiza conexiones de red y transferencia de datos de los procesos",
            Color::Blue
        ),
        (
            " 📝 Ver Reportes (R) ",
            "Consulta los reportes generados de análisis anteriores",
            Color::Magenta
        ),
        (
            " ℹ️ Ayuda (H) ",
            "Muestra información de ayuda sobre cómo usar la aplicación",
            Color::Gray
        ),
        (
            " 🚪 Salir (Q) ",
            "Salir de la aplicación",
            Color::Red
        ),
    ];
    
    // Dibujar cada opción de menú en su columna
    for (i, chunk) in horizontal_chunks.iter().enumerate() {
        let items_per_column = (menu_items.len() + horizontal_chunks.len() - 1) / horizontal_chunks.len();
        let start_idx = i * items_per_column;
        let end_idx = (start_idx + items_per_column).min(menu_items.len());
        
        let mut column_items = Vec::new();
        
        for j in start_idx..end_idx {
            let (title, desc, color) = menu_items[j];
            
            column_items.push(Line::from(vec![
                Span::styled(format!("[{: >3}] ", j + 1), Style::default().fg(Color::White)),
                Span::styled(title, Style::default().fg(color))
            ]));
            
            column_items.push(Line::from(vec![
                Span::styled(desc, Style::default().fg(Color::White))
            ]));
            
            // Espacio entre opciones
            column_items.push(Line::from(vec![Span::raw("")]));
        }
        
        let menu_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));
        
        let menu = Paragraph::new(column_items)
            .block(menu_block)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(menu, *chunk);
    }
} 
