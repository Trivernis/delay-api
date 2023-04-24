use thiserror::Error;

pub type BahnResult<T> = Result<T, BahnError>;

#[derive(Debug, Error)]
pub enum BahnError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
}
