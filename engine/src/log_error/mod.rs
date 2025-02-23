use std::sync::atomic::{AtomicBool, Ordering};

/// Configuration for the logging system
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub log_level: LogLevel,
    pub enable_file_logging: bool,
    pub enable_console_logging: bool,
    pub file_path: Option<String>,
    pub max_file_size: usize,
    pub rotation_count: usize,
}

/// Log levels with numeric values for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

// Re-exports
pub use self::error::*;

#[cfg(debug_assertions)]
mod debug {
    use super::*;
    use crate::log_error::LogConfig;
    use log::{debug, error, info, trace, warn};

    pub fn init(config: LogConfig) -> Result<(), Error> {
        if INITIALIZED.swap(true, Ordering::SeqCst) {
            return Err(Error::AlreadyInitialized);
        }

        // Initialize logging based on config
        let mut builder = env_logger::Builder::from_default_env();

        builder.filter_level(match config.log_level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        });

        if config.enable_file_logging {
            // File logging setup would go here
        }

        builder.init();
        info!("Logging system initialized in debug mode");
        Ok(())
    }

    pub fn log(level: LogLevel, target: &str, message: &str) {
        match level {
            LogLevel::Error => error!(target: target, "{}", message),
            LogLevel::Warn => warn!(target: target, "{}", message),
            LogLevel::Info => info!(target: target, "{}", message),
            LogLevel::Debug => debug!(target: target, "{}", message),
            LogLevel::Trace => trace!(target: target, "{}", message),
        }
    }

    pub fn shutdown() {
        if INITIALIZED.swap(false, Ordering::SeqCst) {
            info!("Logging system shutdown");
        }
    }
}

#[cfg(not(debug_assertions))]
mod release {
    use super::*;
    use crate::log_error::LogConfig;

    #[inline(always)]
    pub fn init(_config: LogConfig) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    pub fn log(_level: LogLevel, _target: &str, _message: &str) {}

    #[inline(always)]
    pub fn shutdown() {}
}

// Export the appropriate implementation based on build configuration
#[cfg(debug_assertions)]
pub use debug::*;

#[cfg(not(debug_assertions))]
pub use release::*;

// Convenience macros that get completely eliminated in release builds
#[macro_export]
macro_rules! log_error {
    ($target:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::log_error::log(
            $crate::log_error::LogLevel::Error,
            $target,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($target:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::log_error::log(
            $crate::log_error::LogLevel::Warn,
            $target,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_info {
    ($target:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::log_error::log(
            $crate::log_error::LogLevel::Info,
            $target,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($target:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::log_error::log(
            $crate::log_error::LogLevel::Debug,
            $target,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_trace {
    ($target:expr, $($arg:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::log_error::log(
            $crate::log_error::LogLevel::Trace,
            $target,
            &format!($($arg)*)
        )
    };
}
