#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ParseDidError {
    #[error("DID {0} method is not the expected one")]
    BadMethod(String),

    #[error("expected a DID instead of an empty string")]
    EmptyString,

    #[error("expected a valid DID syntax")]
    InvalidSyntax,

    #[error("DID lengths exceeds atp 2048 characters limit")]
    TooLong,

    #[error("DID syntax is valid but cannot be used with the current method")]
    Unprocessable,
}

impl From<crate::handle::ParseHandleError> for ParseDidError {
    #[inline(always)]
    fn from(_: crate::handle::ParseHandleError) -> Self {
        Self::Unprocessable
    }
}
