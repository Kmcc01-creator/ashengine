use env_logger::Builder;
use log::{Level, LevelFilter};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub use log::{debug, error, info, warn};

// Sequence number for tracking log message order
static LOG_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct LogContext {
    pub file: &'static str,
    pub line: u32,
    pub function: &'static str,
    pub sequence: u64,
    pub timestamp: u64,
}

impl LogContext {
    pub fn new(file: &'static str, line: u32, function: &'static str) -> Self {
        Self {
            file,
            line,
            function,
            sequence: LOG_SEQUENCE.fetch_add(1, Ordering::SeqCst),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
        }
    }
}

/// Initialize the logging system with custom formatting
pub fn init_logging() {
    let mut builder = Builder::from_default_env();

    builder
        .format(|buf, record| {
            let context = record
                .key_values()
                .get("context")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            writeln!(
                buf,
                "[{} {} {}:{}] {} {}",
                record.level(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                context,
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}

// Macro for logging with context
#[macro_export]
macro_rules! log_with_context {
    ($level:expr, $context:expr, $($arg:tt)+) => {
        log::log!(
            $level,
            target: "physics",
            "{} [{}:{}] {}",
            $context,
            file!(),
            line!(),
            format_args!($($arg)+)
        );
    };
}

// Helper macros for different log levels with context
#[macro_export]
macro_rules! error_with_context {
    ($context:expr, $($arg:tt)+) => {
        $crate::log_with_context!(log::Level::Error, $context, $($arg)+)
    };
}

#[macro_export]
macro_rules! warn_with_context {
    ($context:expr, $($arg:tt)+) => {
        $crate::log_with_context!(log::Level::Warn, $context, $($arg)+)
    };
}

#[macro_export]
macro_rules! info_with_context {
    ($context:expr, $($arg:tt)+) => {
        $crate::log_with_context!(log::Level::Info, $context, $($arg)+)
    };
}

#[macro_export]
macro_rules! debug_with_context {
    ($context:expr, $($arg:tt)+) => {
        $crate::log_with_context!(log::Level::Debug, $context, $($arg)+)
    };
}

// Error chain tracking
pub fn log_error_chain<E: std::error::Error>(
    error: &E,
    context: &str,
    file: &'static str,
    line: u32,
) {
    let mut current = Some(error as &dyn std::error::Error);
    let mut depth = 0;

    error_with_context!(context, "Error occurred at {}:{}", file, line);

    while let Some(err) = current {
        error_with_context!(context, "{}Caused by: {}", "  ".repeat(depth), err);
        current = err.source();
        depth += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::fmt;

    #[derive(Debug)]
    struct TestError(String);

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Error for TestError {}

    #[test]
    fn test_error_chain_logging() {
        init_logging();

        let inner_error = TestError("Inner error".to_string());
        let outer_error = TestError(format!("Outer error: {}", inner_error));

        log_error_chain(&outer_error, "TEST", file!(), line!());
    }
}
