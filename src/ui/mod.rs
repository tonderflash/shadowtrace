pub mod app;
pub mod tui;
pub mod widgets;
pub mod screens;
pub mod events;
pub mod braille_art;

pub use app::App;
pub use tui::Tui;

// Reexportación para fácil acceso
pub use braille_art::{AnimationType, BrailleAnimator, BrailleCanvas, Canvas};
pub use widgets::{
    animated_text::{AnimatedText, AnimatedTextState, ScannerText, ScannerTextState},
    braille_chart::{Axis, BrailleChart, Dataset},
    sparkline_braille::SparklineBraille,
}; 
