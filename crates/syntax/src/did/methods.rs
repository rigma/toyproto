use super::errors::ParseDidError;
use crate::handle::Handle;
use base32ct::{Base32, Encoding};
use regex::Regex;
use smol_str::SmolStr;
use std::sync::LazyLock;

static DID_WEB_LOCALHOST_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^localhost(?:%3A([0-9]{1,5}))$").expect("invalid syntax"));

pub trait Method {
    fn name() -> &'static str;
    fn validate_identifier(identifier: impl AsRef<str>) -> Result<(), ParseDidError>;

    fn parse(did: impl AsRef<str>) -> Result<SmolStr, ParseDidError> {
        let did = did.as_ref();
        if !super::REGEX.is_match(did) {
            return Err(ParseDidError::InvalidSyntax);
        } else if did.len() > super::MAX_LENGTH {
            return Err(ParseDidError::TooLong);
        }

        let prefix = format!("did:{}:", Self::name());
        if did.starts_with(&prefix) {
            let identifier = did[prefix.len()..].to_lowercase();
            Self::validate_identifier(&identifier)?;

            Ok(SmolStr::new(identifier))
        } else {
            Err(ParseDidError::BadMethod(
                did.split(':')
                    .nth(1)
                    .map(|method| method.into())
                    .unwrap_or_default(),
            ))
        }
    }
}

#[derive(Debug)]
pub struct Plc;

impl Method for Plc {
    #[inline(always)]
    fn name() -> &'static str {
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

#[derive(Debug)]
pub struct Web;

impl Method for Web {
    #[inline(always)]
    fn name() -> &'static str {
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
