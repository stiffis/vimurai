use std::error::Error;

/// Result type for vimurai operations
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
