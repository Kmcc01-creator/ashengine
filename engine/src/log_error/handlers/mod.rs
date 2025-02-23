mod console;
mod file;

pub use console::ConsoleHandler;
pub use file::FileHandler;

use crate::log_error::{Error, FormattedLog, LogFormatter, Result};

/// Common trait for all log handlers
pub trait LogHandler: Send + Sync {
    fn write_log(&self, log: &FormattedLog) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

// Console handler implementation
#[derive(Default)]
pub struct ConsoleHandler;

impl LogHandler for ConsoleHandler {
    #[cfg(debug_assertions)]
    fn write_log(&self, log: &FormattedLog) -> Result<()> {
        println!("{}", log);
        Ok(())
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn write_log(&self, _log: &FormattedLog) -> Result<()> {
        Ok(())
    }

    fn flush(&self) -> Result<()> {
        Ok(())
    }
}
