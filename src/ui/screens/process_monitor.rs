use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::{self, Marker},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph, 
        Sparkline, Wrap
    },
    text::{Span, Line},
    Frame,
};

use crate::ui::App;
use crate::ui::braille_art::{BrailleAnimator, AnimationType};

pub fn draw_process_monitor(frame: &mut Frame, app: &mut App) {
    let size = frame.area();
    
    // Dividir la pantalla en secciones
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Título
            Constraint::Min(10),     // Contenido principal
            Constraint::Length(3),   // Barra de estado
        ].as_ref())
        .split(size);
    
    // Título
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Monitoreo de Procesos", 
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    ]))
    .alignment(ratatui::layout::Alignment::Center)
    .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Blue)));
    
    frame.render_widget(title, main_chunks[0]);
    
    // Contenido principal
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Lista de procesos
            Constraint::Percentage(70), // Detalles/gráficos o análisis
        ].as_ref())
        .split(main_chunks[1]);
    
    // Lista de procesos (siempre visible)
    draw_process_list(frame, app, content_chunks[0]);
    
    // Pestañas en el área derecha
    let tabs = vec![
        Span::styled("Detalles", 
            if app.process_monitor_tab == 0 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            }),
        Span::raw(" | "),
        Span::styled("Análisis LLM", 
            if app.process_monitor_tab == 1 {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            }),
    ];
    
    let tabs_row = Line::from(tabs);
    let tabs_para = Paragraph::new(tabs_row)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::TOP | Borders::LEFT | Borders::RIGHT));
    
    // Dividir el área derecha para mostrar las pestañas y el contenido
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Pestañas
            Constraint::Min(5),    // Contenido de la pestaña
        ].as_ref())
        .split(content_chunks[1]);
    
    frame.render_widget(tabs_para, right_chunks[0]);
    
    // Mostrar el contenido según la pestaña seleccionada
    match app.process_monitor_tab {
        0 => {
            // Vista de detalles y gráficos
            let details_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40), // Detalles del proceso
                    Constraint::Percentage(60), // Gráficos
                ].as_ref())
                .split(right_chunks[1]);
            
            draw_process_details(frame, app, details_chunks[0]);
            draw_process_graphs(frame, app, details_chunks[1]);
        },
        1 => {
            // Vista de análisis LLM
            draw_llm_analysis(frame, app, right_chunks[1]);
        },
        _ => unreachable!(),
    }
    
    // Barra de estado con teclas actualizadas
    let status = if let Some(msg) = &app.status_message {
        format!("{}", msg)
    } else {
        "Seleccione un proceso para monitorear".to_string()
    };
    
    // Determinar qué teclas mostrar basado en el estado actual
    let monitoring_controls = if app.is_monitoring_active {
        format!("S: Detener | ")
    } else if app.selected_pid.is_some() {
        let duration_info = if app.monitoring_duration > 0 {
            format!("{}s", app.monitoring_duration)
        } else {
            "∞".to_string()
        };
        format!("M: Iniciar ({}) | 0-5: Duración | ", duration_info)
    } else {
        String::new()
    };
    
    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(" ⌨️ ", Style::default().fg(Color::LightYellow)),
        Span::raw("ESC: Volver | "),
        Span::styled("↑↓", Style::default().fg(Color::LightYellow)),
        Span::raw(": Navegar | "),
        Span::styled("TAB", Style::default().fg(Color::LightYellow)),
        Span::raw(": Cambiar pestaña | "),
        Span::styled("ENTER", Style::default().fg(Color::LightYellow)),
        Span::raw(": Seleccionar | "),
        Span::styled(monitoring_controls, Style::default().fg(Color::LightGreen)),
        Span::styled(" 📋 ", Style::default().fg(Color::LightYellow)),
        Span::raw(format!(": {}", status)),
    ]))
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default());
    
    frame.render_widget(status_bar, main_chunks[2]);
}

fn draw_process_list(frame: &mut Frame, app: &mut App, area: Rect) {
    // Crear lista de procesos
    let processes = &app.processes;
    
    let items: Vec<ListItem> = processes
        .iter()
        .map(|p| {
            let name = p.name.clone();
            let pid = p.pid;
            let cpu = p.cpu_usage;
            
            let content = Line::from(vec![
                Span::raw(format!("{:6} ", pid)),
                Span::styled(
                    format!("{:.1}% ", cpu),
                    Style::default().fg(if cpu > 50.0 { Color::Red } else if cpu > 20.0 { Color::Yellow } else { Color::Green })
                ),
                Span::raw(name),
            ]);
            
            ListItem::new(content)
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Procesos "))
        .highlight_style(Style::default().fg(Color::Black).bg(Color::LightGreen))
        .highlight_symbol(" 👉 ");
    
    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_process_details(frame: &mut Frame, app: &mut App, area: Rect) {
    let selected_pid = app.selected_pid;
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Detalles del Proceso ")
        .style(Style::default().fg(Color::Blue));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    if let Some(pid) = selected_pid {
        if let Some(process) = app.process_monitor.get_process_by_pid(pid) {
            // Detalles del proceso
            let details = vec![
                Line::from(vec![
                    Span::styled("PID:       ", Style::default().fg(Color::LightYellow)),
                    Span::raw(format!("{}", process.pid)),
                ]),
                Line::from(vec![
                    Span::styled("Nombre:    ", Style::default().fg(Color::LightYellow)),
                    Span::raw(process.name.clone()),
                ]),
                Line::from(vec![
                    Span::styled("CPU:       ", Style::default().fg(Color::LightYellow)),
                    Span::styled(
                        format!("{:.2}%", process.cpu_usage),
                        Style::default().fg(
                            if process.cpu_usage > 50.0 { Color::Red } 
                            else if process.cpu_usage > 20.0 { Color::Yellow } 
                            else { Color::Green }
                        ),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Memoria:   ", Style::default().fg(Color::LightYellow)),
                    Span::raw(format!("{} KB", process.memory_usage)),
                ]),
                Line::from(vec![Span::raw("")]),
                Line::from(vec![
                    Span::styled("Ruta:      ", Style::default().fg(Color::LightYellow)),
                    Span::raw(process.path.clone().unwrap_or_else(|| "-".to_string())),
                ]),
                Line::from(vec![
                    Span::styled("Comandos:  ", Style::default().fg(Color::LightYellow)),
                    Span::raw(process.cmd_line.clone().map_or("-".to_string(), |cmd| cmd.join(" "))),
                ]),
            ];
            
            let details_paragraph = Paragraph::new(details)
                .style(Style::default())
                .wrap(Wrap { trim: true });
            
            frame.render_widget(details_paragraph, inner_area);
        } else {
            // No se encontró el proceso
            let text = vec![
                Line::from(vec![
                    Span::styled("Proceso no encontrado", Style::default().fg(Color::Red)),
                ]),
                Line::from(vec![
                    Span::raw(format!("El proceso con PID {} ya no está disponible.", pid)),
                ]),
            ];
            
            let paragraph = Paragraph::new(text)
                .style(Style::default())
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: true });
            
            frame.render_widget(paragraph, inner_area);
        }
    } else {
        // Mensaje de selección de proceso
        let text = vec![
            Line::from(vec![
                Span::styled("Seleccione un proceso", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("Use las flechas ↑↓ para seleccionar un proceso de la lista."),
            ]),
            Line::from(vec![
                Span::raw("Presione ENTER para comenzar a monitorear el proceso seleccionado."),
            ]),
        ];
        
        let paragraph = Paragraph::new(text)
            .style(Style::default())
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, inner_area);
    }
}

fn draw_process_graphs(frame: &mut Frame, app: &mut App, area: Rect) {
    let selected_pid = app.selected_pid;
    
    // Dividir área para los gráficos
    let graphs_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // CPU chart
            Constraint::Percentage(50), // Memory chart
        ].as_ref())
        .split(area);
    
    if let Some(pid) = selected_pid {
        if let Some(process) = app.process_monitor.get_process_by_pid(pid) {
            // Preparar datos para los gráficos
            let cpu_data: Vec<(f64, f64)>;
            let mem_data: Vec<(f64, f64)>;
            
            // Usar datos históricos reales si hay monitoreo activo
            if app.is_monitoring_active && !app.cpu_history.is_empty() {
                // Convertir historial a formato de datos para el gráfico
                cpu_data = app.cpu_history.iter().enumerate()
                    .map(|(i, &value)| (i as f64, value as f64))
                    .collect();
                
                mem_data = app.memory_history.iter().enumerate()
                    .map(|(i, &value)| (i as f64, value as f64 / 1000.0)) // Convertir a MB
                    .collect();
            } else {
                // Usar datos simulados si no hay monitoreo activo
                cpu_data = simulate_chart_data(app.tick_count, process.cpu_usage as f64);
                mem_data = simulate_chart_data(app.tick_count, process.memory_usage as f64 / 1000.0); // Convertir a MB
            }
            
            // Añadir indicadores de monitoreo si está activo
            let mut cpu_title = " CPU % ".to_string();
            let mut mem_title = " Memoria (MB) ".to_string();
            
            if app.is_monitoring_active {
                let elapsed = app.monitoring_time.as_secs();
                let duration_info = if app.monitoring_duration > 0 {
                    format!("{}/{} seg", elapsed, app.monitoring_duration)
                } else {
                    format!("{} seg", elapsed)
                };
                
                cpu_title = format!(" CPU % [Monitoreo: {}] ", duration_info);
                mem_title = format!(" Memoria (MB) [Muestras: {}] ", app.cpu_history.len());
            }
            
            // Gráfico de CPU
            let cpu_dataset = Dataset::default()
                .name("CPU %")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&cpu_data);
            
            let cpu_chart = Chart::new(vec![cpu_dataset])
                .block(Block::default().title(cpu_title).borders(Borders::ALL))
                .x_axis(Axis::default()
                    .title(Span::styled("Tiempo", Style::default().fg(Color::Gray)))
                    .bounds([0.0, if app.is_monitoring_active && !app.cpu_history.is_empty() { 
                        app.cpu_history.len() as f64 
                    } else { 
                        30.0 
                    }])
                    .labels(["0s", "10s", "20s", "30s"]
                        .iter()
                        .map(|&x| Span::raw(x))
                        .collect::<Vec<_>>()))
                .y_axis(Axis::default()
                    .title(Span::styled("CPU %", Style::default().fg(Color::Gray)))
                    .bounds([0.0, 100.0])
                    .labels(["0%", "25%", "50%", "75%", "100%"]
                        .iter()
                        .map(|&x| Span::raw(x))
                        .collect::<Vec<_>>()));
            
            frame.render_widget(cpu_chart, graphs_chunks[0]);
            
            // Gráfico de Memoria
            let mem_dataset = Dataset::default()
                .name("Memoria (MB)")
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Magenta))
                .data(&mem_data);
            
            // Calcular límite máximo para el eje Y de memoria
            let max_mem = if app.is_monitoring_active && !app.memory_history.is_empty() {
                // Usar el valor máximo del historial multiplicado por 1.2 para dar espacio
                let max_val = *app.memory_history.iter().max().unwrap_or(&process.memory_usage);
                (max_val as f64 / 1000.0) * 1.2
            } else {
                (process.memory_usage as f64 / 1000.0) * 1.2
            }.max(10.0); // Mínimo 10 MB para evitar gráficos planos
            
            let mem_chart = Chart::new(vec![mem_dataset])
                .block(Block::default().title(mem_title).borders(Borders::ALL))
                .x_axis(Axis::default()
                    .title(Span::styled("Tiempo", Style::default().fg(Color::Gray)))
                    .bounds([0.0, if app.is_monitoring_active && !app.memory_history.is_empty() { 
                        app.memory_history.len() as f64 
                    } else { 
                        30.0 
                    }])
                    .labels(["0s", "10s", "20s", "30s"]
                        .iter()
                        .map(|&x| Span::raw(x))
                        .collect::<Vec<_>>()))
                .y_axis(Axis::default()
                    .title(Span::styled("MB", Style::default().fg(Color::Gray)))
                    .bounds([0.0, max_mem])
                    .labels(["0", &format!("{:.0}", max_mem/4), &format!("{:.0}", max_mem/2), 
                             &format!("{:.0}", max_mem*3/4), &format!("{:.0}", max_mem)]
                        .iter()
                        .map(|x| Span::raw(x.clone()))
                        .collect::<Vec<_>>()));
            
            frame.render_widget(mem_chart, graphs_chunks[1]);
        } else {
            // Si no hay proceso, mostrar bloques vacíos
            let cpu_block = Block::default()
                .title(" CPU % ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray));
            
            let mem_block = Block::default()
                .title(" Memoria (MB) ")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray));
            
            frame.render_widget(cpu_block, graphs_chunks[0]);
            frame.render_widget(mem_block, graphs_chunks[1]);
        }
    } else {
        // Animación de pulso braille cuando no hay proceso seleccionado
        let block = Block::default()
            .title(" Seleccione un proceso para ver estadísticas ")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Gray));
        
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        
        // Crear una animación de braille
        let braille_width = inner_area.width as usize * 2;
        let braille_height = inner_area.height as usize * 4;
        
        let mut animator = BrailleAnimator::new(braille_width, braille_height, AnimationType::Pulse);
        animator.update(None);
        
        let animation_text = animator.render();
        let animation_paragraph = Paragraph::new(animation_text);
        
        frame.render_widget(animation_paragraph, inner_area);
    }
}

// Función auxiliar para simular datos de gráfico
fn simulate_chart_data(seed: u64, current_value: f64) -> Vec<(f64, f64)> {
    let mut data = Vec::new();
    let phase = (seed % 100) as f64 / 100.0;
    
    for i in 0..30 {
        let x = i as f64;
        // Simulación de valores con variación sinusoidal alrededor del valor actual
        let factor = 0.3 * (x * 0.2 + phase).sin() + 0.7;
        let y = current_value * factor;
        data.push((x, y));
    }
    
    data
}

/// Dibujar panel de análisis LLM
fn draw_llm_analysis(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" Análisis de Comportamiento (IA) ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Blue));
    
    let inner_area = block.inner(area);
    frame.render_widget(block, area);
    
    if let Some(pid) = app.selected_pid {
        if let Some(analysis) = &app.process_llm_analysis {
            // Mostrar el análisis
            let mut formatted_lines = Vec::new();
            
            // Convertir el texto del análisis a un formato compatible con la UI
            for line in analysis.lines() {
                if line.starts_with("##") {
                    // Título principal
                    formatted_lines.push(Line::from(
                        Span::styled(line.trim_start_matches('#').trim(), 
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    ));
                } else if line.starts_with("**") && line.ends_with("**") {
                    // Texto en negrita
                    let text = line.trim_start_matches("**").trim_end_matches("**");
                    formatted_lines.push(Line::from(
                        Span::styled(text, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                    ));
                } else if line.starts_with("- ") {
                    // Elemento de lista
                    formatted_lines.push(Line::from(vec![
                        Span::styled(" • ", Style::default().fg(Color::LightCyan)),
                        Span::raw(line.trim_start_matches("- ")),
                    ]));
                } else if line.starts_with("*") && line.ends_with("*") {
                    // Texto en cursiva
                    let text = line.trim_start_matches('*').trim_end_matches('*');
                    formatted_lines.push(Line::from(
                        Span::styled(text, Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))
                    ));
                } else if line.is_empty() {
                    // Línea vacía
                    formatted_lines.push(Line::from(""));
                } else {
                    // Texto normal
                    formatted_lines.push(Line::from(line));
                }
            }
            
            let analysis_paragraph = Paragraph::new(formatted_lines)
                .style(Style::default())
                .wrap(Wrap { trim: true })
                .scroll((0, 0));
            
            frame.render_widget(analysis_paragraph, inner_area);
        } else {
            // No hay análisis disponible
            let text = vec![
                Line::from(vec![
                    Span::styled("Análisis no disponible", Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::raw("No se ha generado un análisis para este proceso."),
                ]),
                Line::from(vec![Span::raw("")]),
                Line::from(vec![
                    Span::raw("Presione "),
                    Span::styled("TAB", Style::default().fg(Color::LightGreen)),
                    Span::raw(" para generar un análisis de demostración."),
                ]),
            ];
            
            let paragraph = Paragraph::new(text)
                .style(Style::default())
                .alignment(ratatui::layout::Alignment::Center)
                .wrap(Wrap { trim: true });
            
            frame.render_widget(paragraph, inner_area);
        }
    } else {
        // No hay proceso seleccionado
        let text = vec![
            Line::from(vec![
                Span::styled("Seleccione un proceso", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw("Seleccione un proceso de la lista para ver su análisis de comportamiento."),
            ]),
        ];
        
        let paragraph = Paragraph::new(text)
            .style(Style::default())
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, inner_area);
    }
} 
