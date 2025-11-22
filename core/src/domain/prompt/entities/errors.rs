use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum PromptError {
    #[error("Prompt not found")]
    NotFound,

    #[error("Internal server error")]
    InternalServerError,

    #[error("Forbidden")]
    Forbidden,

    #[error("Realm not found")]
    RealmNotFound,

    #[error("Invalid version format")]
    InvalidVersion,
}
