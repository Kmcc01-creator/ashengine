use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static CONTEXT_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct LogContext {
    pub id: u64,
    pub module: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub timestamp: u64,
    pub thread_id: u64,
}

impl LogContext {
    #[cfg(debug_assertions)]
    pub fn new(module: &'static str, file: &'static str, line: u32) -> Self {
        Self {
            id: CONTEXT_ID.fetch_add(1, Ordering::SeqCst),
            module,
            file,
            line,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64,
            thread_id: std::thread::current().id().as_u64().unwrap_or(0),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub fn new(_module: &'static str, _file: &'static str, _line: u32) -> Self {
        unreachable!()
    }

    #[cfg(debug_assertions)]
    pub fn format(&self) -> String {
        format!(
            "[{:016x}] [{:016x}] [{}:{}]",
            self.id, self.thread_id, self.file, self.line
        )
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub fn format(&self) -> String {
        unreachable!()
    }
}

// Convenience macro for creating a LogContext
#[macro_export]
macro_rules! log_context {
    () => {
        #[cfg(debug_assertions)]
        {
            $crate::log_error::context::LogContext::new(module_path!(), file!(), line!())
        }
    };
}

// Helper functions for extracting thread information
#[cfg(debug_assertions)]
mod thread_info {
    use std::thread;

    pub fn current_thread_name() -> String {
        thread::current().name().unwrap_or("unnamed").to_string()
    }

    pub fn current_thread_id() -> u64 {
        thread::current().id().as_u64().unwrap_or(0)
    }
}

#[cfg(not(debug_assertions))]
mod thread_info {
    #[inline(always)]
    pub fn current_thread_name() -> String {
        unreachable!()
    }

    #[inline(always)]
    pub fn current_thread_id() -> u64 {
        unreachable!()
    }
}

pub use thread_info::*;
