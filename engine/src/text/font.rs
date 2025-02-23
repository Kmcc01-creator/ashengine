use crate::error::Result;
use fontdue::Font;
use std::collections::HashMap;
use std::fs::read;
use std::path::Path;
use std::sync::Arc;

pub struct FontManager {
    fonts: HashMap<String, Arc<Font>>,
    default_font: Option<Arc<Font>>,
}

impl FontManager {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            default_font: None,
        }
    }

    pub fn load_font<P: AsRef<Path>>(&mut self, name: &str, path: P) -> Result<()> {
        let font_data = read(path).map_err(|e| {
            VulkanError::ConfigurationError(format!("Failed to read font file: {}", e))
        })?;

        let font = Font::from_bytes(font_data.as_slice(), fontdue::FontSettings::default())
            .map_err(|e| {
                VulkanError::ConfigurationError(format!("Failed to parse font data: {}", e))
            })?;

        let font = Arc::new(font);
        if self.default_font.is_none() {
            self.default_font = Some(Arc::clone(&font));
        }
        self.fonts.insert(name.to_string(), font);
        Ok(())
    }

    pub fn get_font(&self, name: &str) -> Option<Arc<Font>> {
        self.fonts
            .get(name)
            .cloned()
            .or_else(|| self.default_font.clone())
    }

    pub fn get_default_font(&self) -> Option<Arc<Font>> {
        self.default_font.clone()
    }

    pub fn generate_sdf_metrics(
        &self,
        font: &Font,
        glyph: char,
        size: f32,
    ) -> Option<(Vec<u8>, fontdue::Metrics)> {
        let (metrics, bitmap) = font.rasterize(glyph, size);

        // Convert to SDF
        let sdf_size = (metrics.width + 2 * SDF_PADDING as usize)
            * (metrics.height + 2 * SDF_PADDING as usize);
        let mut sdf = vec![0u8; sdf_size];

        // Basic 8-bit SDF generation
        // Note: This is a simplified SDF generation. For production,
        // you might want to use more sophisticated algorithms
        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let idx = y * metrics.width + x;
                if idx < bitmap.len() {
                    let dist = compute_distance(&bitmap, x, y, metrics.width, metrics.height);
                    let sdf_x = x + SDF_PADDING as usize;
                    let sdf_y = y + SDF_PADDING as usize;
                    let sdf_idx = sdf_y * (metrics.width + 2 * SDF_PADDING as usize) + sdf_x;
                    if sdf_idx < sdf.len() {
                        sdf[sdf_idx] = ((dist + 1.0) * 127.5) as u8;
                    }
                }
            }
        }

        Some((sdf, metrics))
    }
}

fn compute_distance(bitmap: &[u8], x: usize, y: usize, width: usize, height: usize) -> f32 {
    let target = bitmap[y * width + x];
    let mut min_dist = f32::MAX;

    // Simple distance field computation
    // Search in a small radius for the nearest different value
    let radius = 3;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let idx = (ny as usize) * width + (nx as usize);
                if idx < bitmap.len() {
                    let sample = bitmap[idx];
                    if (sample > 127) != (target > 127) {
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        min_dist = min_dist.min(dist);
                    }
                }
            }
        }
    }

    if target > 127 {
        min_dist
    } else {
        -min_dist
    }
}

use crate::error::VulkanError;
use crate::text::SDF_PADDING;
