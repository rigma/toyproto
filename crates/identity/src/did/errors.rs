use crate::did::registry::ResolveError;
use reqwest::StatusCode;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResolveDidError {
    #[error("Unable to decoded DID document")]
    DecodeError,

    #[error("DID is not available in directory")]
    DidNotAvailable,

    #[error("DID is not registered in directory")]
    DidNotRegistered,

    #[error("DNS hostname is not registered: {0:?}")]
    HostnameNotRegistered(Option<String>),

    #[error("DID resolution has failed")]
    ResolutionFailed,
}

impl From<ResolveError> for ResolveDidError {
    #[inline]
    fn from(value: ResolveError) -> Self {
        match value {
            ResolveError::NotAvailable => Self::DidNotAvailable,
            ResolveError::NotFound => Self::DidNotRegistered,
            _ => Self::ResolutionFailed,
        }
    }
}

impl From<reqwest::Error> for ResolveDidError {
    #[inline]
    fn from(value: reqwest::Error) -> Self {
        [
            value.is_dns().then_some(Self::HostnameNotRegistered(
                value
                    .url()
                    .and_then(|url| url.host_str())
                    .map(|host| host.into()),
            )),
            value
                .is_status()
                .then(|| match value.status().unwrap_or_default() {
                    StatusCode::NOT_FOUND => Self::DidNotRegistered,
                    StatusCode::GONE => Self::DidNotAvailable,
                    _ => Self::ResolutionFailed,
                }),
        ]
        .into_iter()
        .flatten()
        .next()
        .unwrap_or(Self::ResolutionFailed)
    }
}
