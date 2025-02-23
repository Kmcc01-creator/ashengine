use super::{ConfigManager, EngineConfig, TextBlocksConfig};
use crate::error::{Result, VulkanError};
use log::{debug, error, info, warn};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};

pub struct ConfigLoader {
    config_manager: Arc<ConfigManager>,
    watcher: Option<RecommendedWatcher>,
    config_paths: RwLock<Vec<PathBuf>>,
}

impl ConfigLoader {
    pub fn new(config_manager: Arc<ConfigManager>) -> Result<Self> {
        info!("Initializing ConfigLoader");
        Ok(Self {
            config_manager,
            watcher: None,
            config_paths: RwLock::new(Vec::new()),
        })
    }

    /// Load a configuration file and register it with the config manager
    pub fn load_config<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        info!("Loading config from path: {:?}", path);

        let canonical_path = if !path.is_absolute() {
            match std::env::current_dir() {
                Ok(dir) => dir.join(path),
                Err(e) => {
                    error!("Failed to get current directory: {}", e);
                    return Err(VulkanError::ConfigurationError(format!(
                        "Failed to get current directory: {}",
                        e
                    )));
                }
            }
        } else {
            path.to_path_buf()
        };

        let mut file = File::open(&canonical_path).map_err(|e| {
            error!("Failed to open config file: {} at {:?}", e, canonical_path);
            VulkanError::ConfigurationError(format!("Failed to open config file: {}", e))
        })?;

        // Determine config type from file extension/name
        let file_name = canonical_path.file_name().map(|n| n.to_string_lossy());
        debug!("Processing config file: {:?}", file_name);

        match file_name.as_deref() {
            Some(name) if name.contains("text_blocks") => {
                info!("Loading TextBlocks config");
                let mut contents = String::new();
                file.read_to_string(&mut contents).map_err(|e| {
                    error!("Failed to read config file: {}", e);
                    VulkanError::ConfigurationError(format!("Failed to read config file: {}", e))
                })?;
                let config: TextBlocksConfig = toml::from_str(&contents).map_err(|e| {
                    error!("Failed to parse text blocks config: {}", e);
                    VulkanError::ConfigurationError(format!(
                        "Failed to parse text blocks config: {}",
                        e
                    ))
                    ))?;
                self.config_manager.register(config);
            }
            Some(name) if name.contains("engine") => {
                info!("Loading Engine config");
                let mut contents = String::new();
                file.read_to_string(&mut contents).map_err(|e| {
                    error!("Failed to read config file: {}", e);
                    VulkanError::ConfigurationError(format!("Failed to read config file: {}", e))
                })?;
                let config: EngineConfig = toml::from_str(&contents).map_err(|e| {
                    error!("Failed to parse engine config: {}", e);
                    VulkanError::ConfigurationError(format!(
                        "Failed to parse engine config: {}",
                        e
                    ))
                })?;
                self.config_manager.register(config);
            }
            _ => {
                error!("Unknown config type for file: {:?}", file_name);
                return Err(VulkanError::ConfigurationError(
                    "Unknown config type".to_string(),
                ));
            }
        }

        // Add to watched paths if hot-reloading is enabled
        if self.watcher.is_some() {
            debug!("Adding config path to watch list: {:?}", canonical_path);
            self.config_paths.write().unwrap().push(canonical_path);
        }

        info!("Successfully loaded and registered config from {:?}", path);
        Ok(())
    }

    /// Enable hot-reloading of configuration files
    pub fn enable_hot_reload(&mut self) -> Result<()> {
        info!("Enabling hot reload for configuration files");
        let (tx, rx) = channel();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if matches!(event.kind, notify::EventKind::Modify(_)) {
                    let _ = tx.send(event);
                }
            }
        })
        .map_err(|e| VulkanError::ConfigurationError(format!("Failed to create watcher: {}", e)))?;

        // Watch all currently loaded config files
        for path in self.config_paths.read().unwrap().iter() {
            debug!("Setting up watch for path: {:?}", path);
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| {
                    error!("Failed to watch config file: {}", e);
                    VulkanError::ConfigurationError(format!("Failed to watch config file: {}", e))
                })?;
        }

        let config_manager = Arc::clone(&self.config_manager);

        // Spawn thread to handle config reloading
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                if let notify::Event {
                    kind: notify::EventKind::Modify(_),
                    paths,
                    ..
                } = event
                {
                    for path in paths {
                        debug!("Config file modified: {:?}", path);
                        if let Ok(contents) = std::fs::read_to_string(&path) {
                            let file_name = path.file_name().and_then(|n| n.to_str());
                            match file_name {
                                Some("text_blocks.toml") => {
                                    if let Ok(new_config) = toml::from_str::<TextBlocksConfig>(&contents) {
                                        info!("Hot reloading TextBlocks config from {:?}", path);
                                        config_manager.register(new_config);
                                    }
                                }
                                Some("engine.toml") => {
                                     if let Ok(new_config) = toml::from_str::<EngineConfig>(&contents) {
                                        info!("Hot reloading Engine config from {:?}", path);
                                        config_manager.register(new_config);
                                    }
                                    }
                                } else {
                                    warn!("Unknown config type modified: {:?}", path);
                                }
                            }
                        }
                    }
                }
            }
        });

        self.watcher = Some(watcher);
        info!("Hot reload enabled successfully");
        Ok(())
    }

    /// Add a new path to watch for changes
    pub fn watch_config<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        if let Some(watcher) = &mut self.watcher {
            debug!("Adding new config path to watch: {:?}", path);
            watcher
                .watch(path, RecursiveMode::NonRecursive)
                .map_err(|e| {
                    error!("Failed to watch config file: {}", e);
                    VulkanError::ConfigurationError(format!("Failed to watch config file: {}", e))
                })?;
            self.config_paths.write().unwrap().push(path.to_owned());
        }
        Ok(())
    }
}

impl Drop for ConfigLoader {
    fn drop(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            for path in self.config_paths.read().unwrap().iter() {
                debug!("Removing watch for path: {:?}", path);
                let _ = watcher.unwatch(path);
            }
        }
    }
}
