//! Error types for labwc-rs using `thiserror`.

use thiserror::Error;

/// Top-level error type for the compositor.
#[derive(Error, Debug)]
pub enum CompositorError {
    /// Wayland display creation failed.
    #[error("failed to create Wayland display")]
    DisplayCreation,

    /// Backend initialization failed.
    #[error("backend initialization failed: {0}")]
    Backend(String),

    /// Renderer initialization failed.
    #[error("renderer initialization failed: {0}")]
    Renderer(String),

    /// Configuration parsing failed.
    #[error("configuration error: {0}")]
    Config(String),

    /// A Wayland protocol error was received.
    #[error("protocol error: {0}")]
    Protocol(String),

    /// An invalid operation was attempted.
    #[error("invalid operation: {0}")]
    InvalidOperation(String),

    /// Resource allocation failed (OOM).
    #[error("allocation failed: {0}")]
    Allocation(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Unknown error.
    #[error("unknown error: {0}")]
    Unknown(String),
}

/// Configuration-specific errors.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// XML parsing failed.
    #[error("XML parse error at line {line}: {msg}")]
    XmlParse {
        /// Line number where the error occurred.
        line: usize,
        /// Error message.
        msg: String,
    },

    /// Missing required configuration key.
    #[error("missing required key: {0}")]
    MissingKey(String),

    /// Invalid configuration value.
    #[error("invalid value for '{key}': {value}")]
    InvalidValue {
        /// The configuration key.
        key: String,
        /// The invalid value.
        value: String,
    },

    /// File not found.
    #[error("configuration file not found: {0}")]
    FileNotFound(String),

    /// Theme error.
    #[error("theme error: {0}")]
    Theme(String),
}

/// Result type alias for compositor operations.
pub type Result<T> = std::result::Result<T, CompositorError>;

/// Result type alias for configuration operations.
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
