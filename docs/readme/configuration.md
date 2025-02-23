# Configuration

This document describes how to configure AshEngine.

AshEngine uses a TOML-based configuration system. Configuration files are loaded and managed by the `ConfigLoader` struct (in `config/loader.rs`). The engine supports hot-reloading of configuration files.

## Configuration Files

Configuration files are loaded based on their file names. Currently, the following configuration files are supported:

- `engine.toml`: Contains general engine settings.

## `engine.toml`

This file contains global engine settings. It has a top-level section `[engine]`.

Example:

```toml
# Engine configuration

[engine]
physics_enabled = false
lighting_enabled = false
```

### Options

- `physics_enabled`: (boolean) Enables or disables the physics engine.
- `lighting_enabled`: (boolean) Enables or disables lighting.

## `ConfigManager`

The `ConfigManager` (in `config/mod.rs`) is responsible for storing and providing access to the loaded configuration data. It uses a `HashMap` to store different configuration types, keyed by their module name.

## `ConfigLoader`

The `ConfigLoader` (in `config/loader.rs`) handles loading configuration files, parsing them, and registering them with the `ConfigManager`. It also supports hot-reloading: if hot-reloading is enabled and a configuration file is modified, the `ConfigLoader` automatically reloads the file and updates the configuration in the `ConfigManager`.

### Hot Reloading

To enable hot-reloading, you need to call `ConfigLoader::enable_hot_reload()`. This sets up a file system watcher that monitors the configuration files for changes. When a change is detected, the file is reloaded and parsed, and the `ConfigManager` is updated.

## Adding New Configuration Types

To add a new configuration type:

1.  Define a struct that represents your configuration data. This struct should derive `Deserialize`, `Clone`, and `Debug`.
2.  Implement the `Config` trait for your struct.
3.  Add a case to the `ConfigLoader::load_config` function to handle loading and parsing your configuration file based on its file name. Use `toml::from_str` to deserialize the TOML data into your struct.
4.  Register the loaded configuration with the `ConfigManager` using `config_manager.register(config)`.
5.  Add support for hot-reloading in the thread spawned in `enable_hot_reload`.
