use ratatui::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::Style,
    widgets::{Block, StatefulWidget, Widget},
};
use crate::ui::braille_art::{BrailleCanvas, Canvas};

/// Un widget de gráfico tipo sparkline usando caracteres braille para mayor resolución
pub struct SparklineBraille<'a> {
    /// Título del gráfico
    title: Option<&'a str>,
    /// Datos a mostrar
    data: &'a [f64],
    /// Estilo del widget
    style: Style,
    /// Bloque contenedor
    block: Option<Block<'a>>,
    /// Valor máximo (si es None, se calcula automáticamente)
    max: Option<f64>,
    /// Valor mínimo (si es None, se calcula automáticamente)
    min: Option<f64>,
}

impl<'a> Default for SparklineBraille<'a> {
    fn default() -> Self {
        Self {
            title: None,
            data: &[],
            style: Style::default(),
            block: None,
            max: None,
            min: None,
        }
    }
}

impl<'a> SparklineBraille<'a> {
    pub fn new(data: &'a [f64]) -> Self {
        Self {
            title: None,
            data,
            style: Style::default(),
            block: None,
            max: None,
            min: None,
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

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
}

impl<'a> Widget for SparklineBraille<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Aplicar bloque si existe
        let chart_area = match self.block {
            Some(ref b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
            None => area,
        };

        if chart_area.width < 1 || chart_area.height < 1 || self.data.is_empty() {
            return;
        }

        // Calcular valores min/max
        let max = self.max.unwrap_or_else(|| {
            self.data.iter().fold(f64::MIN, |acc, &x| acc.max(x))
        });
        let min = self.min.unwrap_or_else(|| {
            self.data.iter().fold(f64::MAX, |acc, &x| acc.min(x))
        });

        // Crear canvas braille (cada carácter braille tiene 2x4 puntos)
        let width = chart_area.width as usize * 2;
        let height = chart_area.height as usize * 4;
        let mut canvas = BrailleCanvas::new(width, height);

        // Dibujar puntos con interpolación si es necesario
        let data_len = self.data.len();
        let x_scale = width as f64 / data_len.max(1) as f64;
        let y_scale = height as f64 / (max - min + 1.0);

        // Dibujar la línea
        for i in 0..data_len - 1 {
            let x1 = (i as f64 * x_scale) as usize;
            let x2 = ((i + 1) as f64 * x_scale) as usize;
            let y1 = height - ((self.data[i] - min) * y_scale) as usize;
            let y2 = height - ((self.data[i + 1] - min) * y_scale) as usize;

            // Limitar a los bordes del canvas
            let y1 = y1.min(height - 1);
            let y2 = y2.min(height - 1);
            let x1 = x1.min(width - 1);
            let x2 = x2.min(width - 1);

            // Dibujar línea entre puntos
            self.draw_line(&mut canvas, x1, y1, x2, y2);
        }

        // Convertir a string y renderizar en el buffer
        let lines = canvas.to_string();
        for (i, line) in lines.lines().enumerate() {
            if i < chart_area.height as usize {
                buf.set_string(
                    chart_area.x,
                    chart_area.y + i as u16,
                    line,
                    self.style,
                );
            }
        }
    }
}

impl<'a> SparklineBraille<'a> {
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
