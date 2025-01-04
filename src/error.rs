#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum GhError {
    #[error("GitHub authentication failed: {0}")]
    AuthError(String),

    #[error("Invalid repository URL: {0}")]
    InvalidRepoUrl(String),

    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("Rate limit exceeded. Reset at: {0}")]
    RateLimitError(String),

    #[error("Invalid output format: {0}")]
    InvalidFormat(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    OctocrabError(#[from] octocrab::Error),

    #[error(transparent)]
    TemplateError(#[from] indicatif::style::TemplateError),
}
