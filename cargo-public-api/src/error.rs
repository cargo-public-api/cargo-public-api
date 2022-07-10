#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The API diff is not allowed as per --deny")]
    DiffDenied,
}
