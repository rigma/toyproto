use reqwest::StatusCode;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResolveError {
    #[error("fetched DID document is not a valid JSON document")]
    InvalidEncoding,

    #[error("DID document is not available in the registry")]
    NotAvailable,

    #[error("DID document was not found in the registry")]
    NotFound,

    #[error("unknown error has occured: {0}")]
    UnknownError(String),
}

impl From<reqwest::Error> for ResolveError {
    #[inline]
    fn from(value: reqwest::Error) -> Self {
        if value.is_status() {
            match value.status().unwrap_or_default() {
                StatusCode::NOT_FOUND => Self::NotFound,
                StatusCode::GONE => Self::NotAvailable,
                _ => Self::UnknownError(format!("{value}")),
            }
        } else if value.is_decode() {
            Self::InvalidEncoding
        } else {
            Self::UnknownError(format!("{value}"))
        }
    }
}
