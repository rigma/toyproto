use hickory_resolver::net::{DnsError, NetError};

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum ResolveNsidError {
    #[error("Lexicon bound to provided DNS was not found")]
    NotFound,

    #[error("NSID Lexicon resolution has failed")]
    ResolutionFailed,
}

impl From<DnsError> for ResolveNsidError {
    #[inline]
    fn from(value: DnsError) -> Self {
        match value {
            DnsError::NoRecordsFound(_) => Self::NotFound,
            _ => Self::ResolutionFailed,
        }
    }
}

impl From<NetError> for ResolveNsidError {
    #[inline]
    fn from(value: NetError) -> Self {
        match value {
            NetError::Dns(error) => From::from(error),
            _ => Self::ResolutionFailed,
        }
    }
}
