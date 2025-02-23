use super::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlocksConfig {
    pub text_settings: TextSettings,
    pub theme: Theme,
    pub blocks: Vec<TextBlock>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextSettings {
    pub default_font: String,
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub sdf_settings: SDFSettings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SDFSettings {
    pub smoothing: f32,
    pub thickness: f32,
    pub padding: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Theme {
    pub colors: HashMap<String, [f32; 4]>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextBlock {
    pub id: String,
    pub content: String,
    pub position: [f32; 2],
    pub color: String,
    pub scale: f32,
    pub selectable: bool,
}

impl TextBlocksConfig {
    pub fn new() -> Self {
        Self {
            text_settings: TextSettings::default(),
            theme: Theme::default(),
            blocks: Vec::new(),
        }
    }
}

impl Config for TextBlocksConfig {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn module_name(&self) -> &str {
        "text_blocks"
    }
}
