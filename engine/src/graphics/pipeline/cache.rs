//! Pipeline cache implementation
//!
//! Provides efficient caching and reuse of pipeline objects with support for
//! serialization and runtime optimization.

use ash::vk;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};

use super::{variants::VariantCache, PipelineVariant};
use crate::error::Result;

/// Cache statistics tracking
#[derive(Debug, Default)]
pub struct CacheStats {
    hits: usize,
    misses: usize,
    evictions: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total > 0 {
            self.hits as f32 / total as f32
        } else {
            0.0
        }
    }
}

/// Pipeline cache controller
pub struct PipelineCache {
    device: Arc<ash::Device>,
    cache: vk::PipelineCache,
    variants: RwLock<VariantCache>,
    stats: RwLock<CacheStats>,
    max_size: usize,
}

impl PipelineCache {
    /// Create a new pipeline cache
    pub fn new(device: Arc<ash::Device>, max_size: usize) -> Result<Self> {
        let cache_info = vk::PipelineCacheCreateInfo::builder();
        let cache = unsafe {
            device
                .create_pipeline_cache(&cache_info, None)
                .map_err(|e| crate::error::VulkanError::PipelineCacheCreation(e.to_string()))?
        };

        Ok(Self {
            device,
            cache,
            variants: RwLock::new(VariantCache::new()),
            stats: RwLock::new(CacheStats::default()),
            max_size,
        })
    }

    /// Get pipeline from cache if it exists
    pub fn get(&self, variant: &PipelineVariant) -> Option<vk::Pipeline> {
        let variants = self.variants.read();
        let pipeline = variants.get(variant);

        if pipeline.is_some() {
            self.stats.write().hits += 1;
        } else {
            self.stats.write().misses += 1;
        }

        pipeline
    }

    /// Insert pipeline into cache
    pub fn insert(&self, variant: PipelineVariant, pipeline: vk::Pipeline) {
        let mut variants = self.variants.write();

        // Simple eviction if we're at capacity
        if variants.variants.len() >= self.max_size {
            if let Some((old_variant, old_pipeline)) = variants
                .variants
                .iter()
                .next()
                .map(|(k, v)| (k.clone(), *v))
            {
                variants.remove(&old_variant);
                unsafe {
                    self.device.destroy_pipeline(old_pipeline, None);
                }
                self.stats.write().evictions += 1;
            }
        }

        variants.insert(variant, pipeline);
    }

    /// Save cache to disk
    pub fn save_to_disk(&self, path: &std::path::Path) -> Result<()> {
        let data = unsafe {
            self.device
                .get_pipeline_cache_data(self.cache)
                .map_err(|e| crate::error::VulkanError::PipelineCacheDataRetrieval(e.to_string()))?
        };

        std::fs::write(path, &data)
            .map_err(|e| crate::error::VulkanError::PipelineCacheDataSave(e.to_string()))?;

        Ok(())
    }

    /// Load cache from disk
    pub fn load_from_disk(&self, path: &std::path::Path) -> Result<()> {
        let data = std::fs::read(path)
            .map_err(|e| crate::error::VulkanError::PipelineCacheDataLoad(e.to_string()))?;

        let create_info = vk::PipelineCacheCreateInfo::builder().initial_data(&data);

        unsafe {
            self.device
                .create_pipeline_cache(&create_info, None)
                .map_err(|e| crate::error::VulkanError::PipelineCacheCreation(e.to_string()))?;
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Clear the cache
    pub fn clear(&self) {
        let mut variants = self.variants.write();
        for (_, pipeline) in variants.variants.drain() {
            unsafe {
                self.device.destroy_pipeline(pipeline, None);
            }
        }
        self.stats.write().evictions += 1;
    }

    /// Get the Vulkan pipeline cache handle
    pub fn vk_cache(&self) -> vk::PipelineCache {
        self.cache
    }
}

impl Drop for PipelineCache {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            self.device.destroy_pipeline_cache(self.cache, None);
        }
    }
}
