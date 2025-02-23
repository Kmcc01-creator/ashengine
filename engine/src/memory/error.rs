use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Failed to allocate memory: {0}")]
    AllocationFailed(String),

    #[error("Memory type {0} not supported")]
    UnsupportedMemoryType(u32),

    #[error("Out of memory: requested {requested} bytes, available {available} bytes")]
    OutOfMemory { requested: u64, available: u64 },

    #[error("Invalid alignment: required {required}, got {got}")]
    InvalidAlignment { required: u64, got: u64 },

    #[error("Buffer too large: {0} bytes")]
    BufferTooLarge(u64),

    #[error("Invalid buffer access: {0}")]
    InvalidBufferAccess(String),

    #[error("Memory mapping failed: {0}")]
    MappingFailed(String),

    #[error("Memory leak detected: {0} bytes not freed")]
    MemoryLeak(u64),

    #[error("Invalid memory operation: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, MemoryError>;
