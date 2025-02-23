use chrono::{DateTime, Local};
use std::fmt;

use crate::log_error::{LogContext, LogLevel};

#[derive(Debug, Clone)]
pub struct FormattedLog {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub context: Option<LogContext>,
    pub message: String,
}

impl FormattedLog {
    #[cfg(debug_assertions)]
    pub fn new(level: LogLevel, context: Option<LogContext>, message: impl Into<String>) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            context,
            message: message.into(),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub fn new(level: LogLevel, _context: Option<LogContext>, message: impl Into<String>) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            context: None,
            message: message.into(),
        }
    }
}

impl fmt::Display for FormattedLog {
    #[cfg(debug_assertions)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = match self.level {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN ",
            LogLevel::Info => "INFO ",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        };

        if let Some(ctx) = &self.context {
            write!(
                f,
                "[{}] [{}] {} {}",
                self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                level_str,
                ctx.format(),
                self.message
            )
        } else {
            write!(
                f,
                "[{}] [{}] {}",
                self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                level_str,
                self.message
            )
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // In release mode, we only output timestamp, level, and message
        write!(
            f,
            "[{}] {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.message
        )
    }
}

pub trait LogFormatter {
    fn format(&self, log: &FormattedLog) -> String;
}

#[derive(Default)]
pub struct DefaultFormatter;

impl LogFormatter for DefaultFormatter {
    #[cfg(debug_assertions)]
    fn format(&self, log: &FormattedLog) -> String {
        log.to_string()
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn format(&self, log: &FormattedLog) -> String {
        log.to_string()
    }
}

#[derive(Default)]
pub struct JsonFormatter;

impl LogFormatter for JsonFormatter {
    #[cfg(debug_assertions)]
    fn format(&self, log: &FormattedLog) -> String {
        serde_json::json!({
            "timestamp": log.timestamp.to_rfc3339(),
            "level": format!("{:?}", log.level),
            "context": log.context.as_ref().map(|ctx| {
                json!({
                    "id": ctx.id,
                    "module": ctx.module,
                    "file": ctx.file,
                    "line": ctx.line,
                    "thread_id": ctx.thread_id
                })
            }),
            "message": log.message
        })
        .to_string()
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    fn format(&self, log: &FormattedLog) -> String {
        serde_json::json!({
            "timestamp": log.timestamp.to_rfc3339(),
            "level": format!("{:?}", log.level),
            "message": log.message
        })
        .to_string()
    }
}
