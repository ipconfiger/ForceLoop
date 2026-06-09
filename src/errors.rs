use thiserror::Error;

#[derive(Error, Debug)]
pub enum ForceLoopError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Execution error: {0}")]
    Execution(String),
}

pub type Result<T> = std::result::Result<T, ForceLoopError>;
