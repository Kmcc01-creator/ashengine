pub mod atlas;
pub mod font;
pub mod layout;
pub mod picking;
pub mod vertex;

pub use atlas::{FontAtlas, GlyphInfo, GlyphMetrics};
pub use font::FontManager;
pub use layout::{BoundingBox, Rect, TextElement, TextLayout};
pub use picking::TextPicker;
pub use vertex::TextVertex;

// Re-export common types and traits
pub trait TextRenderable {
    fn to_text_elements(&self) -> Vec<TextElement>;
}

// Default font settings
pub const DEFAULT_FONT_SIZE: f32 = 16.0;
pub const DEFAULT_LINE_HEIGHT: f32 = 1.2;
pub const DEFAULT_LETTER_SPACING: f32 = 0.0;

// SDF rendering constants
pub const SDF_SMOOTHING: f32 = 0.125;
pub const SDF_THICKNESS: f32 = 0.5;
pub const SDF_PADDING: f32 = 4.0;

// Text alignment options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

// Helper functions
pub fn calculate_text_bounds(text: &str, font_size: f32, line_height: f32) -> (f32, f32) {
    let char_count = text.chars().count() as f32;
    let width = char_count * font_size * 0.6; // Approximate width
    let height = font_size * line_height;
    (width, height)
}

pub fn pixel_to_ndc(x: f32, y: f32, width: f32, height: f32) -> [f32; 2] {
    [(x / width) * 2.0 - 1.0, (y / height) * 2.0 - 1.0]
}

pub fn ndc_to_pixel(x: f32, y: f32, width: f32, height: f32) -> [f32; 2] {
    [(x + 1.0) * width * 0.5, (y + 1.0) * height * 0.5]
}

// Error type for text-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error("Font not found: {0}")]
    FontNotFound(String),

    #[error("Failed to load glyph: {0}")]
    GlyphLoadError(String),

    #[error("Invalid text layout: {0}")]
    LayoutError(String),
}

// Result type alias
pub type TextResult<T> = Result<T, TextError>;

// Configuration structures
#[derive(Debug, Clone)]
pub struct TextConfig {
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub alignment: TextAlignment,
    pub color: [f32; 4],
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            font_size: DEFAULT_FONT_SIZE,
            line_height: DEFAULT_LINE_HEIGHT,
            letter_spacing: DEFAULT_LETTER_SPACING,
            alignment: TextAlignment::Left,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}
