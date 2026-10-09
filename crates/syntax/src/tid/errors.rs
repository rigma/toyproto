use std::{sync::PoisonError, time::SystemTimeError};

#[derive(Debug, thiserror::Error)]
pub(super) enum DecodeError {
    #[error("empty string provided to the decoder")]
    EmptyString,

    #[error("invalid symbol '{symbol}' at position {position}")]
    InvalidSymbol { symbol: char, position: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum FromPartsError {
    #[error("provided clock ID is not a 10 bits word")]
    UnboundClockId,

    #[error("provided timestamp is greater than Tid::MAX_TIMESTAMP")]
    UnboundTimestamp,
}

#[derive(Debug, thiserror::Error)]
pub enum NextTidError {
    #[error("unable to generate timestamp from UNIX epoch")]
    InvalidTimestamp(#[from] SystemTimeError),

    #[error("current thread is already owning a mutex of the default clock")]
    PoisonedClock,

    #[error("TID produced by the clock is bound")]
    UnboundTid(#[from] FromPartsError),
}

impl<T> From<PoisonError<T>> for NextTidError {
    #[inline]
    fn from(_: PoisonError<T>) -> Self {
        Self::PoisonedClock
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseTidError {
    #[error("unable to decode TID: invalid symbol '{symbol}' at position {position}")]
    DecodeError { symbol: char, position: usize },

    #[error("provided TID has not a valid syntax")]
    InvalidSyntax,
}

impl From<DecodeError> for ParseTidError {
    fn from(value: DecodeError) -> Self {
        match value {
            DecodeError::EmptyString => Self::InvalidSyntax,
            DecodeError::InvalidSymbol { symbol, position } => {
                Self::DecodeError { symbol, position }
            }
        }
    }
}
