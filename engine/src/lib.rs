//! AshEngine - A Vulkan-based graphics engine written in Rust

pub mod config;
pub mod graphics;
pub mod lighting;
pub mod log_error;
pub mod memory;
pub mod physics;
pub mod text;

// Re-exports for convenience
pub use log_error::{
    log_debug, log_error, log_info, log_trace, log_warn, Error as EngineError, LogConfig, LogLevel,
    Result,
};

pub use graphics::{Pipeline, RenderPass, Renderer, Swapchain};

// Re-export all the types needed for text rendering
pub use text::{
    ndc_to_pixel, pixel_to_ndc, FontAtlas, TextAlignment, TextConfig, TextElement, TextLayout,
    TextPicker,
};
