use reqwest::StatusCode;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResolveHandleError {
    #[error("Handle cannot be found")]
    NotFound,

    #[error("provided handle resolution has failed")]
    ResolutionFailed,

    #[error("handle has been resolved but cannot be decoded into a DID")]
    Unprocessable,
}

impl From<hickory_resolver::net::DnsError> for ResolveHandleError {
    #[inline(always)]
    fn from(value: hickory_resolver::net::DnsError) -> Self {
        use hickory_resolver::net::DnsError;

        match value {
            DnsError::NoRecordsFound(_) => Self::NotFound,
            _ => Self::ResolutionFailed,
        }
    }
}

impl From<hickory_resolver::net::NetError> for ResolveHandleError {
    #[inline]
    fn from(value: hickory_resolver::net::NetError) -> Self {
        use hickory_resolver::net::NetError;

        match value {
            NetError::Dns(error) => From::from(error),
            _ => Self::ResolutionFailed,
        }
    }
}

impl From<reqwest::Error> for ResolveHandleError {
    #[inline]
    fn from(value: reqwest::Error) -> Self {
        [
            value.is_dns().then_some(Self::NotFound),
            value.is_status().then(|| {
                if value.status().unwrap_or_default() == StatusCode::NOT_FOUND {
                    Self::NotFound
                } else {
                    Self::ResolutionFailed
                }
            }),
            value.is_decode().then_some(Self::Unprocessable),
        ]
        .into_iter()
        .flatten()
        .next()
        .unwrap_or(Self::ResolutionFailed)
    }
}

impl From<toyproto_syntax::did::ParseDidError> for ResolveHandleError {
    #[inline]
    fn from(_: toyproto_syntax::did::ParseDidError) -> Self {
        Self::Unprocessable
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidateHandleError {
    #[error("handle TLD is not valid")]
    InvalidTld,

    #[error("handle syntax is not valid")]
    InvalidSyntax,
}
