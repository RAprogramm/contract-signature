use thiserror::Error;

#[derive(Debug, Error)]
pub enum SigError {
    #[error("DOM not available")]
    DomUnavailable,
    #[error("Element not found: {0}")]
    ElementNotFound(String),
    #[error("Canvas context unavailable")]
    NoContext2d,
    #[error("Operation failed: {0}")]
    OpFailed(String)
}
