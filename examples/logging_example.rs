use ashengine::log_error::{
    self, log_debug, log_error, log_info, log_trace, log_warn, LogConfig, LogLevel,
};
use std::path::PathBuf;

fn main() {
    // Initialize logging with debug configuration
    let config = LogConfig {
        log_level: LogLevel::Debug,
        enable_file_logging: true,
        enable_console_logging: true,
        file_path: Some("logs/engine.log".into()),
        max_file_size: 1024 * 1024, // 1MB
        rotation_count: 5,
    };

    // Initialize the logging system
    if let Err(e) = log_error::init(config) {
        eprintln!("Failed to initialize logging: {}", e);
        return;
    }

    // Example usage of different log levels
    log_info!("system", "Application started");

    log_debug!("memory", "Allocating buffer of size: {}", 1024);

    log_error!("graphics", "Failed to create texture: {}", "Invalid format");

    log_warn!(
        "physics",
        "Collision detection taking longer than expected: {}ms",
        50
    );

    log_trace!("input", "Mouse moved to position ({}, {})", 100, 200);

    // Example with context
    {
        let _ctx = log_context!();
        log_error!("test", "This error includes file and line context");
    }

    // Nested operations with logging
    perform_operation("test_operation").unwrap_or_else(|e| {
        log_error!("main", "Operation failed: {}", e);
    });
}

fn perform_operation(name: &str) -> Result<(), String> {
    log_info!("operation", "Starting operation: {}", name);

    // Simulate some work
    std::thread::sleep(std::time::Duration::from_millis(100));

    log_debug!("operation", "Operation completed: {}", name);
    Ok(())
}
