mod text_blocks;

use serde::Deserialize;
pub use text_blocks::*;

#[derive(Deserialize, Clone, Debug)]
pub struct EngineConfig {
    pub engine: EngineSettings,
}

#[derive(Deserialize, Clone, Debug)]
pub struct EngineSettings {
    pub physics_enabled: bool,
    pub lighting_enabled: bool,
}

impl Config for EngineConfig {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn module_name(&self) -> &str {
        "engine"
    }
}

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub trait Config: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn module_name(&self) -> &str;
}

#[derive(Default)]
pub struct ConfigManager {
    configs: RwLock<HashMap<String, Arc<RwLock<dyn Config>>>>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
        }
    }

    pub fn register<T: Config + 'static>(&self, config: T) {
        let module_name = config.module_name().to_string();
        let config = Arc::new(RwLock::new(config));
        self.configs.write().unwrap().insert(module_name, config);
    }

    pub fn get<T: Config + Clone + 'static>(&self, module_name: &str) -> Option<T> {
        self.configs
            .read()
            .unwrap()
            .get(module_name)
            .and_then(|config| {
                config.read().ok().and_then(|guard| {
                    guard
                        .as_any()
                        .downcast_ref::<T>()
                        .map(|config| config.clone())
                })
            })
    }
}

mod loader;
pub use loader::*;
