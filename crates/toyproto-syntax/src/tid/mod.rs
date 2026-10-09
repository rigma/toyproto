mod base32_sortable;
mod clock;
pub mod errors;

use clock::DefaultClock;
pub use clock::TidClock;
use errors::{FromPartsError, NextTidError, ParseTidError};
use regex::Regex;
use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, Serializer},
};
use std::{str::FromStr, sync::LazyLock};

const CLOCK_ID_MASK: u64 = 0x3ff;

static TID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^[2-7a-j][2-7a-z]{12}$").expect("invalid syntax"));
static CLOCK: LazyLock<DefaultClock> = LazyLock::new(|| DefaultClock::new());

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Tid(u64);

impl Tid {
    const MAX_TIMESTAMP: u64 = (1 << 53) - 1;

    #[inline]
    pub fn from_parts(timestamp: u64, clock_id: u16) -> Result<Self, FromPartsError> {
        if timestamp > Self::MAX_TIMESTAMP {
            Err(FromPartsError::UnboundTimestamp)
        } else if clock_id > CLOCK_ID_MASK as u16 {
            Err(FromPartsError::UnboundClockId)
        } else {
            Ok(Self((timestamp << 10) | (clock_id as u64)))
        }
    }

    #[inline]
    pub fn next() -> Result<Self, NextTidError> {
        CLOCK.next_tid()
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    #[inline]
    pub fn clock_id(&self) -> u16 {
        (self.0 & CLOCK_ID_MASK) as u16
    }

    #[inline]
    pub fn timestamp(&self) -> u64 {
        self.0 >> 10
    }

    #[inline]
    pub fn to_string(&self) -> String {
        base32_sortable::encode(self.0)
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.to_string()
    }
}

impl FromStr for Tid {
    type Err = ParseTidError;

    #[inline]
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if TID_REGEX.is_match(raw) {
            Ok(Self(base32_sortable::decode(raw)?))
        } else {
            Err(Self::Err::InvalidSyntax)
        }
    }
}

impl PartialEq<Box<[u8]>> for Tid {
    #[inline]
    fn eq(&self, other: &Box<[u8]>) -> bool {
        base32_sortable::decode(other).map_or(false, |other| self.0 == other)
    }
}

impl PartialEq<String> for Tid {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        base32_sortable::decode(other).map_or(false, |other| self.0 == other)
    }
}

impl PartialEq<[u8]> for Tid {
    #[inline]
    fn eq(&self, other: &[u8]) -> bool {
        base32_sortable::decode(other).map_or(false, |other| self.0 == other)
    }
}

impl PartialEq<str> for Tid {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        base32_sortable::decode(other).map_or(false, |other| self.0 == other)
    }
}

impl PartialEq<u64> for Tid {
    #[inline]
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl TryFrom<Box<[u8]>> for Tid {
    type Error = ParseTidError;

    #[inline]
    fn try_from(value: Box<[u8]>) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value[..])
    }
}

impl TryFrom<String> for Tid {
    type Error = ParseTidError;

    #[inline]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<&[u8]> for Tid {
    type Error = ParseTidError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        String::from_utf8_lossy(value).parse()
    }
}

impl TryFrom<[u8; 13]> for Tid {
    type Error = ParseTidError;

    #[inline]
    fn try_from(value: [u8; 13]) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value[..])
    }
}

impl TryFrom<&str> for Tid {
    type Error = ParseTidError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl TryFrom<u64> for Tid {
    type Error = FromPartsError;

    #[inline]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_parts(value >> 10, (value & CLOCK_ID_MASK) as u16)
    }
}

impl<'de> Deserialize<'de> for Tid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TidVisitor;
        impl<'de> de::Visitor<'de> for TidVisitor {
            type Value = Tid;

            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "expecting a base32-sortable string")
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(|err| E::custom(format!("{err}")))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(|err| E::custom(format!("{err}")))
            }

            #[inline]
            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_string(self)
            }
        }

        deserializer.deserialize_newtype_struct("Tid", TidVisitor)
    }
}

impl Serialize for Tid {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("Tid", &self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_tokens};
    use std::assert_matches;

    const CANONICAL_VALID_TIDS: [(Tid, &'static str); 4] = [
        // 2222222222222
        (Tid(0), "2222222222222"),
        // 7777777777777
        (Tid(0x5294a5294a5294a5), "7777777777777"),
        // 3zzzzzzzzzzzz
        (Tid(0x1fffffffffffffff), "3zzzzzzzzzzzz"),
        // 3jzfcijpj2z2a
        (Tid(0x17fd6873eaf07c06), "3jzfcijpj2z2a"),
    ];

    #[test]
    fn it_supports_serde_data_model() {
        for (tid, repr) in CANONICAL_VALID_TIDS {
            assert_tokens(
                &tid,
                &[Token::NewtypeStruct { name: "Tid" }, Token::String(repr)],
            );
        }
    }

    #[test]
    fn it_raises_an_error_on_invalid_tids() {
        assert_matches!(
            "3jzfcijpj2z21".parse::<Tid>(),
            Err(ParseTidError::InvalidSyntax)
        );
        assert_matches!(
            "0000000000000".parse::<Tid>(),
            Err(ParseTidError::InvalidSyntax)
        );
        assert_matches!(
            "3JZFCIJPJ2Z2A".parse::<Tid>(),
            Err(ParseTidError::InvalidSyntax)
        );
        assert_matches!(
            "3jzfcijpj2z2aa".parse::<Tid>(),
            Err(ParseTidError::InvalidSyntax)
        );
        assert_matches!(
            "3jzfcijpj2z2".parse::<Tid>(),
            Err(ParseTidError::InvalidSyntax)
        );
        assert_matches!("222".parse::<Tid>(), Err(ParseTidError::InvalidSyntax));
        assert_matches!(
            "3jzf-cij-pj2z-2a".parse::<Tid>(),
            Err(ParseTidError::InvalidSyntax)
        );
        assert_matches!(
            "zzzzzzzzzzzzz".parse::<Tid>(),
            Err(ParseTidError::InvalidSyntax)
        );
        assert_matches!(
            "kjzfcijpj2z2a".parse::<Tid>(),
            Err(ParseTidError::InvalidSyntax)
        );
    }
}
