use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Block, Widget},
};
use crate::ui::braille_art::{BrailleCanvas, Canvas};

/// Un widget de gráfico de alta resolución usando caracteres braille
pub struct BrailleChart<'a> {
    /// Block para el widget
    block: Option<Block<'a>>,
    /// Conjunto de datos a mostrar
    datasets: Vec<Dataset<'a>>,
    /// Estilo del widget
    style: Style,
    /// Configuración del eje X
    x_axis: Axis<'a>,
    /// Configuración del eje Y
    y_axis: Axis<'a>,
    /// Título del gráfico
    title: Option<&'a str>,
}

/// Conjunto de datos para el gráfico
pub struct Dataset<'a> {
    /// Nombre del conjunto de datos
    name: &'a str,
    /// Puntos de datos (x, y)
    data: Vec<(f64, f64)>,
    /// Estilo del conjunto de datos
    style: Style,
}

/// Configuración de un eje
pub struct Axis<'a> {
    /// Título del eje
    title: Option<Span<'a>>,
    /// Etiquetas del eje
    labels: Vec<Span<'a>>,
    /// Límites del eje
    bounds: [f64; 2],
    /// Estilo del eje
    style: Style,
}

impl<'a> Default for Axis<'a> {
    fn default() -> Self {
        Self {
            title: None,
            labels: Vec::new(),
            bounds: [0.0, 0.0],
            style: Style::default(),
        }
    }
}

impl<'a> Axis<'a> {
    pub fn title<S>(mut self, title: S) -> Self
    where
        S: Into<Span<'a>>,
    {
        self.title = Some(title.into());
        self
    }

    pub fn labels<T>(mut self, labels: T) -> Self
    where
        T: Into<Vec<Span<'a>>>,
    {
        self.labels = labels.into();
        self
    }

    pub fn bounds(mut self, bounds: [f64; 2]) -> Self {
        self.bounds = bounds;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Dataset<'a> {
    pub fn new<S>(name: S, data: Vec<(f64, f64)>) -> Self
    where
        S: Into<&'a str>,
    {
        Self {
            name: name.into(),
            data,
            style: Style::default(),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Default for BrailleChart<'a> {
    fn default() -> Self {
        Self {
            block: None,
            datasets: Vec::new(),
            style: Style::default(),
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            title: None,
        }
    }
}

impl<'a> BrailleChart<'a> {
    pub fn new<D>(datasets: D) -> Self
    where
        D: Into<Vec<Dataset<'a>>>,
    {
        Self {
            block: None,
            datasets: datasets.into(),
            style: Style::default(),
            x_axis: Axis::default(),
            y_axis: Axis::default(),
            title: None,
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn x_axis(mut self, axis: Axis<'a>) -> Self {
        self.x_axis = axis;
        self
    }

    pub fn y_axis(mut self, axis: Axis<'a>) -> Self {
        self.y_axis = axis;
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
}

impl<'a> Widget for BrailleChart<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 5 || area.height < 5 {
            return;
        }

        // Aplicar bloque si existe
        let chart_area = match self.block {
            Some(ref b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
            None => area,
        };

        // Dimensiones para ejes y etiquetas
        let y_label_width = 6; // Ancho para etiquetas del eje Y
        let x_label_height = 2; // Alto para etiquetas del eje X

        // Calcular área del gráfico restando espacio para ejes y etiquetas
        let graph_area = Rect {
            x: chart_area.x + y_label_width,
            y: chart_area.y,
            width: chart_area.width.saturating_sub(y_label_width),
            height: chart_area.height.saturating_sub(x_label_height),
        };

        if graph_area.width == 0 || graph_area.height == 0 {
            return;
        }

        // Dibujar ejes y etiquetas
        self.render_axes(chart_area, graph_area, buf);

        // Crear canvas Braille para el gráfico
        let canvas_width = graph_area.width as usize * 2; // 2 puntos por carácter en X
        let canvas_height = graph_area.height as usize * 4; // 4 puntos por carácter en Y
        let mut canvas = BrailleCanvas::new(canvas_width, canvas_height);

        // Dibujar cada conjunto de datos
        for dataset in &self.datasets {
            self.draw_dataset(dataset, &mut canvas, &graph_area);
        }

        // Convertir canvas a string y renderizar
        let canvas_str = canvas.to_string();
        let lines: Vec<&str> = canvas_str.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if i < graph_area.height as usize {
                buf.set_string(
                    graph_area.x,
                    graph_area.y + i as u16,
                    line,
                    self.style,
                );
            }
        }
    }
}

impl<'a> BrailleChart<'a> {
    fn render_axes(&self, chart_area: Rect, graph_area: Rect, buf: &mut Buffer) {
        // Dibujar eje Y
        let y_axis_x = graph_area.x - 1;
        for y in 0..graph_area.height {
            buf.set_string(y_axis_x, graph_area.y + y, "│", self.style);
        }

        // Dibujar eje X
        let x_axis_y = graph_area.y + graph_area.height;
        for x in 0..graph_area.width {
            buf.set_string(graph_area.x + x, x_axis_y, "─", self.style);
        }

        // Dibujar intersección de ejes
        buf.set_string(y_axis_x, x_axis_y, "┼", self.style);

        // Dibujar etiquetas del eje Y
        if !self.y_axis.labels.is_empty() {
            let label_count = self.y_axis.labels.len();
            for (i, label) in self.y_axis.labels.iter().enumerate() {
                let y_pos = graph_area.y + graph_area.height - 
                    (i as f32 / (label_count - 1) as f32 * graph_area.height as f32) as u16;
                
                if y_pos < chart_area.y + chart_area.height {
                    let label_x = chart_area.x;
                    buf.set_string(label_x, y_pos, "      ", self.style); // Limpiar espacio
                    buf.set_span(label_x, y_pos, label, label.content.len().min(6) as u16);
                    buf.set_string(y_axis_x, y_pos, "┤", self.style); // Marca en el eje
                }
            }
        }

        // Dibujar etiquetas del eje X
        if !self.x_axis.labels.is_empty() {
            let label_count = self.x_axis.labels.len();
            for (i, label) in self.x_axis.labels.iter().enumerate() {
                let x_pos = graph_area.x + 
                    (i as f32 / (label_count - 1) as f32 * graph_area.width as f32) as u16;
                
                if x_pos < chart_area.x + chart_area.width {
                    let label_y = x_axis_y + 1;
                    buf.set_span(x_pos.saturating_sub(label.content.len() as u16 / 2), 
                                label_y, label, label.content.len() as u16);
                    buf.set_string(x_pos, x_axis_y, "┬", self.style); // Marca en el eje
                }
            }
        }

        // Dibujar título del eje Y
        if let Some(title) = &self.y_axis.title {
            let y_title_x = chart_area.x;
            let y_title_y = graph_area.y;
            buf.set_span(y_title_x, y_title_y, title, title.content.len().min(6) as u16);
        }

        // Dibujar título del eje X
        if let Some(title) = &self.x_axis.title {
            let x_title_x = graph_area.x + graph_area.width / 2;
            let x_title_y = chart_area.y + chart_area.height - 1;
            buf.set_span(
                x_title_x.saturating_sub(title.content.len() as u16 / 2),
                x_title_y,
                title,
                title.content.len() as u16,
            );
        }
    }

    fn draw_dataset(&self, dataset: &Dataset<'a>, canvas: &mut BrailleCanvas, area: &Rect) {
        if dataset.data.is_empty() {
            return;
        }

        // Obtener límites
        let x_min = self.x_axis.bounds[0];
        let x_max = self.x_axis.bounds[1];
        let y_min = self.y_axis.bounds[0];
        let y_max = self.y_axis.bounds[1];

        // Escalar a coordenadas del canvas
        let width = canvas.width();
        let height = canvas.height();
        
        let scale_x = |x: f64| -> usize {
            ((x - x_min) / (x_max - x_min) * width as f64) as usize
        };
        
        let scale_y = |y: f64| -> usize {
            height - ((y - y_min) / (y_max - y_min) * height as f64) as usize
        };

        // Dibujar puntos y líneas
        let mut prev_x = None;
        let mut prev_y = None;

        for &(x, y) in &dataset.data {
            if x < x_min || x > x_max || y < y_min || y > y_max {
                prev_x = None;
                prev_y = None;
                continue;
            }

            let canvas_x = scale_x(x).min(width - 1);
            let canvas_y = scale_y(y).min(height - 1);

            // Dibujar punto
            canvas.set(canvas_x, canvas_y, true);

            // Dibujar línea al punto anterior
            if let (Some(px), Some(py)) = (prev_x, prev_y) {
                self.draw_line(canvas, px, py, canvas_x, canvas_y);
            }

            prev_x = Some(canvas_x);
            prev_y = Some(canvas_y);
        }
    }

    // Algoritmo de Bresenham para dibujar líneas
    fn draw_line(&self, canvas: &mut BrailleCanvas, x0: usize, y0: usize, x1: usize, y1: usize) {
        let mut x0 = x0 as isize;
        let mut y0 = y0 as isize;
        let x1 = x1 as isize;
        let y1 = y1 as isize;

        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let width = canvas.width() as isize;
        let height = canvas.height() as isize;

        loop {
            if x0 >= 0 && x0 < width && y0 >= 0 && y0 < height {
                canvas.set(x0 as usize, y0 as usize, true);
            }
            
            if x0 == x1 && y0 == y1 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 >= dy {
                if x0 == x1 {
                    break;
                }
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                if y0 == y1 {
                    break;
                }
                err += dx;
                y0 += sy;
            }
        }
    }
} 
