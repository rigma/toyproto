use crate::{did::errors::ResolveDidError, handle::errors::ResolveHandleError};

#[derive(Debug, thiserror::Error)]
pub enum BuilderError {
    #[error("unable to instanciate directory DNS client")]
    DnsClientError(#[from] hickory_resolver::net::NetError),

    #[error("unable to instanciate directory HTTP client")]
    HttpClientError(#[from] reqwest::Error),

    #[error("provided user agent cannot be parsed as a HTTP header value")]
    InvalidUserAgent,
}

impl From<reqwest::header::InvalidHeaderValue> for BuilderError {
    #[inline]
    fn from(_: reqwest::header::InvalidHeaderValue) -> Self {
        Self::InvalidUserAgent
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum LookupError {
    #[error("lookup has failed because DID cannot be resolved")]
    DidResolution(#[from] ResolveDidError),

    #[error("lookup has failed because handle and DID are not matching")]
    HandleMismatch { declared: String, given: String },

    #[error("lookup has failed because handle cannot be resolved")]
    HandleResolution(#[from] ResolveHandleError),

    #[error("lookup was not able to check handle and DID match")]
    UncheckedHandle,
}
