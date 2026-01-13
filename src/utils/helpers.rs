use thiserror::Error;

/// Errors related to data types and value conversions
#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Type conversion error: {0}")]
    ConversionError(String),
    #[error("Unsupported data type: {0}")]
    UnsupportedType(String),
    #[error("Invalid value for type {0}: {1}")]
    InvalidValue(String, String),
    #[error("Value comparison error: {0}")]
    ComparisonError(String),
}
