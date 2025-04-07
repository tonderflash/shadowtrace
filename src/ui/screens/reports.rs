use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Line},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::ui::App;
use crate::ui::braille_art::{BrailleAnimator, AnimationType};

pub fn draw_reports(frame: &mut Frame, app: &mut App) {
    let size = frame.area();
    
    // Dividir la pantalla en secciones
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Título
            Constraint::Min(10),     // Contenido principal
            Constraint::Length(3),   // Barra de estado
        ].as_ref())
        .split(size);
    
    // Título
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Reportes de Monitoreo", 
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
    ]))
    .alignment(ratatui::layout::Alignment::Center)
    .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Blue)));
    
    frame.render_widget(title, chunks[0]);
    
    // En construcción - Mostrar una animación
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Visor de Reportes en Construcción ")
        .style(Style::default().fg(Color::Magenta));
    
    let inner_area = block.inner(chunks[1]);
    frame.render_widget(block, chunks[1]);
    
    // Animación
    let braille_width = inner_area.width as usize * 2;
    let braille_height = inner_area.height as usize * 4;
    
    let mut animator = BrailleAnimator::new(braille_width, braille_height, AnimationType::Spiral);
    animator.update(None);
    
    let animation_text = animator.render();
    let animation_paragraph = Paragraph::new(animation_text);
    
    frame.render_widget(animation_paragraph, inner_area);
    
    // Barra de estado
    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(" ⌨️ ", Style::default().fg(Color::LightYellow)),
        Span::raw("ESC: Volver | "),
        Span::styled("↑↓", Style::default().fg(Color::LightYellow)),
        Span::raw(": Navegar | "),
        Span::styled("ENTER", Style::default().fg(Color::LightYellow)),
        Span::raw(": Ver reporte"),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default());
    
    frame.render_widget(status_bar, chunks[2]);
} 
