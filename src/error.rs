use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
/// Enumerates all errors that can currently occur within this crate.
pub enum Error {
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

/// Shorthand for [`std::result::Result<T, public_items::Error>`].
pub type Result<T> = std::result::Result<T, Error>;
