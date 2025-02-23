use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Logging system already initialized")]
    AlreadyInitialized,

    #[error("Logging system not initialized")]
    NotInitialized,

    #[error("Invalid log level: {0}")]
    InvalidLogLevel(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to create log file: {0}")]
    FileCreation(String),

    #[error("Failed to write to log file: {0}")]
    FileWrite(String),

    #[error("Failed to rotate log files: {0}")]
    FileRotation(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("System error: {0}")]
    System(String),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

// Result type alias for convenience
pub type Result<T> = std::result::Result<T, Error>;

// Error conversion implementations
impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::System(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::System(s.to_string())
    }
}

// Helper function to create configuration errors
pub fn config_err(msg: impl Into<String>) -> Error {
    Error::Configuration(msg.into())
}

// Helper function to create system errors
pub fn system_err(msg: impl Into<String>) -> Error {
    Error::System(msg.into())
}

#[cfg(debug_assertions)]
pub(crate) fn debug_err(msg: impl Into<String>) -> Error {
    Error::System(format!("Debug error: {}", msg.into()))
}

// Production builds will completely eliminate this code
#[cfg(not(debug_assertions))]
#[inline(always)]
pub(crate) fn debug_err(_msg: impl Into<String>) -> Error {
    unreachable!()
}
