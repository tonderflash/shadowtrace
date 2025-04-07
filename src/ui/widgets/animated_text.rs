use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, StatefulWidget, Widget, Paragraph},
};
use std::time::{Duration, Instant};

use crate::ui::braille_art::{AnimationType, BrailleAnimator};

/// Tipos de animación para el texto
pub enum AnimationStyle {
    /// Efecto de escritura (typewriter)
    Typing,
    /// Colores pulsantes
    Pulse,
    /// Efecto arcoíris
    Rainbow,
    /// Texto parpadeante
    Blink,
    /// Sin animación
    None,
}

/// Estado del widget de texto animado
pub struct AnimatedTextState {
    /// Contador de cuadros
    frames: usize,
    /// Último momento de actualización
    last_update: Instant,
    /// Intervalo de actualización
    update_interval: Duration,
    /// Tipo de animación actual
    animation_type: AnimationType,
}

impl Default for AnimatedTextState {
    fn default() -> Self {
        Self {
            frames: 0,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
            animation_type: AnimationType::Wave,
        }
    }
}

impl AnimatedTextState {
    /// Crear un nuevo estado para texto animado
    pub fn new() -> Self {
        Self::default()
    }

    /// Establecer el intervalo de actualización
    pub fn update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }

    /// Establecer el tipo de animación
    pub fn set_animation_type(mut self, animation_type: AnimationType) -> Self {
        self.animation_type = animation_type;
        self
    }

    /// Actualizar el estado (llamar en cada ciclo)
    pub fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.frames = self.frames.wrapping_add(1);
            self.last_update = now;
        }
    }

    /// Obtener el cuadro actual
    pub fn frame(&self) -> usize {
        self.frames
    }

    /// Obtener el tipo de animación actual
    pub fn get_animation_type(&self) -> AnimationType {
        self.animation_type
    }
}

/// Widget de texto con efectos de animación
pub struct AnimatedText<'a> {
    /// Texto a mostrar
    text: Text<'a>,
    /// Estilo base
    style: Style,
    /// Estilo de la animación
    animation_style: AnimationStyle,
    /// Bloque contenedor
    block: Option<Block<'a>>,
    /// Repetir la animación
    repeat: bool,
}

impl<'a> AnimatedText<'a> {
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            text: text.into(),
            style: Style::default(),
            animation_style: AnimationStyle::None,
            block: None,
            repeat: false,
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

    pub fn animation_style(mut self, style: AnimationStyle) -> Self {
        self.animation_style = style;
        self
    }

    pub fn repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }
}

impl<'a> StatefulWidget for AnimatedText<'a> {
    type State = AnimatedTextState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner_area = match self.block {
            Some(b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
            None => area,
        };

        if inner_area.width < 1 || inner_area.height < 1 {
            return;
        }

        // Actualizar el estado
        state.update();

        // Crear y actualizar el animador
        let mut animator = BrailleAnimator::new(
            self.text.lines.iter().fold(0, |acc, line| {
                acc + line.spans.iter().fold(0, |line_acc, span| {
                    line_acc + span.content.chars().count()
                })
            }) * 2,  // El doble de caracteres en x para braille
            self.text.lines.len() * 4,  // 4 veces en y para braille
            state.get_animation_type(),
        );
        animator.update(Some(state.frame()));

        // Convertir el texto a string y crear texto con la animación
        let text_str = self.text.lines.iter()
            .map(|line| {
                line.spans.iter()
                    .map(|span| span.content.clone().to_string())
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect::<Vec<String>>()
            .join("\n");

        // Renderizar el texto y la animación
        let text_with_animation = format!(
            "{}\n{}",
            animator.render(),
            text_str
        );

        // Dividir en líneas y renderizar
        let lines: Vec<&str> = text_with_animation.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if i < inner_area.height as usize {
                buf.set_string(
                    inner_area.x,
                    inner_area.y + i as u16,
                    line,
                    self.style,
                );
            }
        }
    }
}

/// Widget de texto con una animación tipo "escáner" que se mueve sobre el texto
pub struct ScannerText<'a> {
    /// Bloque contenedor
    block: Option<Block<'a>>,
    /// Texto a mostrar
    text: Text<'a>,
    /// Estilo del widget
    style: Style,
    /// Estilo de la animación
    scanner_style: Style,
}

/// Estado para ScannerText
pub struct ScannerTextState {
    /// Posición actual del escáner
    position: usize,
    /// Dirección del movimiento
    direction: isize,
    /// Último momento de actualización
    last_update: Instant,
    /// Intervalo de actualización
    update_interval: Duration,
}

impl Default for ScannerTextState {
    fn default() -> Self {
        Self {
            position: 0,
            direction: 1,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(150),
        }
    }
}

impl ScannerTextState {
    /// Crear un nuevo estado para el escáner
    pub fn new() -> Self {
        Self::default()
    }

    /// Establecer el intervalo de actualización
    pub fn update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }

    /// Actualizar el estado (llamar en cada ciclo)
    pub fn update(&mut self, max_width: usize) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            self.position = (self.position as isize + self.direction) as usize;
            
            // Cambiar dirección en los bordes
            if self.position >= max_width - 3 {
                self.direction = -1;
            } else if self.position == 0 {
                self.direction = 1;
            }
            
            self.last_update = now;
        }
    }
}

impl<'a> ScannerText<'a> {
    /// Crear un nuevo widget de texto con escáner
    pub fn new<T>(text: T) -> Self
    where
        T: Into<Text<'a>>,
    {
        Self {
            block: None,
            text: text.into(),
            style: Style::default(),
            scanner_style: Style::default(),
        }
    }

    /// Establecer el bloque
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Establecer el estilo
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Establecer el estilo del escáner
    pub fn scanner_style(mut self, style: Style) -> Self {
        self.scanner_style = style;
        self
    }
}

impl<'a> StatefulWidget for ScannerText<'a> {
    type State = ScannerTextState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let inner_area = match self.block {
            Some(b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
            None => area,
        };

        if inner_area.width < 1 || inner_area.height < 1 {
            return;
        }

        // Actualizar la posición del escáner
        state.update(inner_area.width as usize);

        // Renderizar el texto
        let text_str = self.text.to_string();
        let lines: Vec<&str> = text_str.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if i < inner_area.height as usize {
                // Renderizar la línea normalmente
                buf.set_string(
                    inner_area.x,
                    inner_area.y + i as u16,
                    line,
                    self.style,
                );

                // Renderizar el efecto de escáner
                if !line.is_empty() {
                    let pos = state.position.min(line.len().saturating_sub(1));
                    let scanner_width = 3.min(inner_area.width as usize - pos);
                    
                    // Extraer la parte a resaltar
                    let highlight = &line[pos..pos + scanner_width.min(line.len() - pos)];
                    
                    // Aplicar el estilo del escáner
                    buf.set_string(
                        inner_area.x + pos as u16,
                        inner_area.y + i as u16,
                        highlight,
                        self.scanner_style,
                    );
                }
            }
        }
    }
} 

