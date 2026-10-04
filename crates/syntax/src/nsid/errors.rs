use crate::handle::ParseHandleError;

#[derive(Debug, thiserror::Error)]
pub enum BuildNsidError {
    #[error("provided domain is not a valid handle")]
    InvalidDomain,

    #[error("provided name is not valid for a NSID")]
    InvalidName,
}

impl From<ParseHandleError> for BuildNsidError {
    #[inline(always)]
    fn from(_: ParseHandleError) -> Self {
        Self::InvalidDomain
    }
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum ParseNsidError {
    #[error("unable to parse empty string into a NSID")]
    EmptyString,

    #[error("provided string is not a valid NSID")]
    InvalidSyntax,

    #[error("provided string is too long to be a valid NSID")]
    TooLong,
}

impl From<ParseHandleError> for ParseNsidError {
    #[inline(always)]
    fn from(_: ParseHandleError) -> Self {
        Self::InvalidSyntax
    }
}
