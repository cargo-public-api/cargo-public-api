use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive] // We reserve the right to add more enum variants
pub enum Error {
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
