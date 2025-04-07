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
    let size = frame.size();
    
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
        msg.clone()
    } else {
        "Seleccione un proceso para monitorear".to_string()
    };
    
    // Crear componentes de la barra de estado con teclas claras y visibles
    let mut status_spans = vec![
        Span::styled(" ⌨️ ", Style::default().fg(Color::LightYellow)),
        Span::raw("ESC: Volver | "),
        Span::styled("↑↓", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
        Span::raw(": Navegar | "),
        Span::styled("ENTER", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
        Span::raw(": Seleccionar | "),
    ];
    
    // Añadir controles específicos basados en el estado actual
    if app.is_monitoring_active {
        status_spans.push(Span::styled("S", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)));
        status_spans.push(Span::raw(": Detener monitoreo | "));
    } else if app.selected_pid.is_some() {
        status_spans.push(Span::styled("M", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)));
        status_spans.push(Span::raw(": Monitorear | "));
        
        // Destacar opción de analizar si hay suficientes datos
        let analyze_style = if app.cpu_history.len() >= 5 {
            Style::default().fg(Color::LightGreen).bg(Color::Black).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)
            };
            
        status_spans.push(Span::styled("A", analyze_style));
        if app.cpu_history.len() >= 5 {
            status_spans.push(Span::raw(": Analizar datos | "));
        } else {
            status_spans.push(Span::raw(": Analizar | "));
        }
        
        status_spans.push(Span::styled("TAB", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)));
        status_spans.push(Span::raw(": Cambiar vista | "));
    }
    
    // Añadir mensaje de estado
    status_spans.push(Span::styled(" 📋 ", Style::default().fg(Color::LightYellow)));
    status_spans.push(Span::raw(format!(": {}", status)));
    
    let status_bar = Paragraph::new(Line::from(status_spans))
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
            
            // Formato mejorado para mayor visibilidad
            let content = Line::from(vec![
                Span::raw(format!("{:<8}", pid)),
                Span::styled(
                    format!("{:>6.1}% ", cpu),
                    Style::default()
                        .fg(if cpu > 50.0 { Color::Red } 
                            else if cpu > 20.0 { Color::Yellow } 
                            else { Color::Green })
                        .add_modifier(Modifier::BOLD)
                ),
                Span::raw(name),
            ]);
            
            ListItem::new(content)
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Procesos ")
            .title_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)))
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
            } else if app.cpu_history.len() >= 5 {
                // Mostrar indicador de datos listos para análisis
                cpu_title = format!(" CPU % [Datos recopilados: {}] ", app.cpu_history.len());
                mem_title = format!(" Memoria (MB) [Análisis disponible ✓] ");
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
            
            // Crear etiquetas para el eje Y como strings para evitar problemas de lifetime
            let label_0 = "0".to_string();
            let label_1 = format!("{:.0}", max_mem/4.0);
            let label_2 = format!("{:.0}", max_mem/2.0);
            let label_3 = format!("{:.0}", max_mem*3.0/4.0);
            let label_4 = format!("{:.0}", max_mem);
            
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
                    .labels([&label_0, &label_1, &label_2, &label_3, &label_4]
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
    // Mostrar análisis LLM si hay uno disponible
    if let Some(analysis) = &app.process_llm_analysis {
        // Convertir el análisis markdown a texto formateado para la interfaz
        let text = convert_markdown_to_spans(analysis);
        
        // Calcular si necesitamos scroll vertical
        let total_lines = text.len();
        let visible_lines = area.height as usize - 2; // Restamos 2 por los bordes
        
        // Inicializar o actualizar el scroll si es necesario
        if app.llm_text_scroll_index.is_none() {
            app.llm_text_scroll_index = Some(0);
        }
        
        // Obtener el índice de scroll actual
        let scroll_index = app.llm_text_scroll_index.unwrap_or(0);
        
        // Mostrar indicadores de scroll solo si es necesario
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(" Análisis LLM ");
            
        // Añadir indicadores de scroll en el título si hay más contenido
        if total_lines > visible_lines {
            let scroll_percentage = if total_lines > 0 {
                (scroll_index as f64 / (total_lines - visible_lines.min(total_lines)) as f64 * 100.0) as u16
            } else {
                0
            };
            
            let scroll_info = format!(" Análisis LLM [{}%] ↓↑ ", scroll_percentage);
            block = block.title(scroll_info);
        }
        
        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((scroll_index as u16, 0));
            
        frame.render_widget(paragraph, area);
        
        // Instrucciones de navegación si hay scroll disponible
        if total_lines > visible_lines {
            let nav_text = "↑/↓: Navegar | PgUp/PgDn: Saltar";
            let nav_style = Style::default().fg(Color::DarkGray);
            
            // Crear un pequeño widget para mostrar las instrucciones de navegación
            let nav_width = nav_text.len() as u16 + 4;
            let nav_height = 1;
            let nav_x = area.x + area.width.saturating_sub(nav_width);
            let nav_y = area.y + area.height.saturating_sub(1);
            
            if nav_x >= area.x && nav_y >= area.y {
                let nav_area = Rect::new(nav_x, nav_y, nav_width, nav_height);
                let nav_widget = Paragraph::new(Line::from(vec![Span::styled(nav_text, nav_style)]))
                    .alignment(ratatui::layout::Alignment::Right);
                
                frame.render_widget(nav_widget, nav_area);
            }
        }
    } else if let Some(pid) = app.selected_pid {
        // Mostrar un mensaje para iniciar análisis
        let mut content = vec![
            Line::from(vec![
                Span::styled("No hay análisis para este proceso", 
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            ]),
            Line::from(""),
        ];
        
        // Restablecer el índice de scroll cuando no hay análisis
        app.llm_text_scroll_index = None;
        
        if app.is_monitoring_active {
            content.push(Line::from(vec![
                Span::raw("El monitoreo está activo. Presiona "),
                Span::styled("S", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" para detenerlo y luego "),
                Span::styled("A", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" para realizar un análisis.")
            ]));
        } else if app.cpu_history.is_empty() {
            content.push(Line::from(vec![
                Span::raw("No hay datos de monitoreo. Presiona "),
                Span::styled("M", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" para monitorear este proceso.")
            ]));
        } else {
            content.push(Line::from(vec![
                Span::raw("Presiona "),
                Span::styled("A", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw(" para realizar un análisis con los datos recopilados.")
            ]));
        }
        
        let paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(" Análisis LLM "))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    } else {
        // Mensaje cuando no hay proceso seleccionado
        let content = vec![
            Line::from(vec![
                Span::styled("Selecciona un proceso para analizar", 
                    Style::default().fg(Color::Gray))
            ]),
        ];
        
        let paragraph = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(" Análisis LLM "))
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        
        frame.render_widget(paragraph, area);
    }
}

// Función para convertir markdown simple a spans con formato
fn convert_markdown_to_spans(markdown: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    
    for line in markdown.lines() {
        // Procesar encabezados, negritas, etc.
        if line.starts_with("##") {
            let title = line.trim_start_matches('#').trim();
            lines.push(Line::from(vec![
                Span::styled(title, 
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            ]));
        } else if line.starts_with("#") {
            let title = line.trim_start_matches('#').trim();
            lines.push(Line::from(vec![
                Span::styled(title, 
                    Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
            ]));
        } else if line.contains("**") {
            // Procesar negritas con formato especial
            let mut spans = Vec::new();
            let mut current_text = String::new();
            let mut is_bold = false;
            
            for part in line.split("**") {
                if !current_text.is_empty() {
                    if is_bold {
                        spans.push(Span::styled(current_text.clone(), 
                            Style::default().add_modifier(Modifier::BOLD)));
                    } else {
                        spans.push(Span::raw(current_text.clone()));
                    }
                    current_text.clear();
                }
                
                current_text = part.to_string();
                is_bold = !is_bold;
            }
            
            if !current_text.is_empty() && !is_bold {
                spans.push(Span::raw(current_text));
            }
            
            lines.push(Line::from(spans));
        } else if line.trim().starts_with("-") || line.trim().starts_with("*") || line.trim().starts_with("•") {
            // Lista con viñetas
            let item_text = line.trim_start_matches('-')
                .trim_start_matches('*')
                .trim_start_matches('•')
                .trim();
            
            lines.push(Line::from(vec![
                Span::styled(" • ", Style::default().fg(Color::Yellow)),
                Span::raw(item_text)
            ]));
        } else if line.is_empty() {
            // Línea en blanco
            lines.push(Line::default());
        } else {
            // Texto normal
            lines.push(Line::from(line));
        }
    }
    
    lines
} 
