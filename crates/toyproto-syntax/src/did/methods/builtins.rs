use super::{Method, MethodName};
use crate::{did::errors::ParseDidError, handle::Handle};
use base32ct::{Base32, Encoding};
use regex::Regex;
use std::sync::LazyLock;

static DID_WEB_LOCALHOST_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^localhost(?:%3A([0-9]{1,5}))$").expect("invalid syntax"));

#[derive(Clone, Debug)]
pub struct Plc;

impl Method for Plc {
    #[inline(always)]
    fn name() -> impl Into<MethodName> {
        "plc"
    }

    #[inline]
    fn validate_identifier(identifier: impl AsRef<str>) -> Result<(), ParseDidError> {
        let identifier = identifier.as_ref();

        if identifier.len() == 24 && Base32::decode_vec(identifier).is_ok() {
            Ok(())
        } else {
            Err(ParseDidError::Unprocessable)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Universal;

impl Method for Universal {
    #[inline(always)]
    fn name() -> impl Into<MethodName> {
        MethodName::Undefined
    }

    #[inline(always)]
    fn validate_identifier(_: impl AsRef<str>) -> Result<(), ParseDidError> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Web;

impl Method for Web {
    #[inline(always)]
    fn name() -> impl Into<MethodName> {
        "web"
    }

    #[inline]
    fn validate_identifier(identifier: impl AsRef<str>) -> Result<(), ParseDidError> {
        let identifier = identifier.as_ref();

        if let Some(captures) = DID_WEB_LOCALHOST_REGEX.captures(identifier) {
            captures
                .get(0)
                .map(|port| {
                    port.as_str()
                        .parse::<u16>()
                        .map(|_| ())
                        .map_err(|_| ParseDidError::Unprocessable)
                })
                .unwrap_or(Ok(()))
        } else {
            identifier.parse::<Handle>()?;
            Ok(())
        }
    }
}
