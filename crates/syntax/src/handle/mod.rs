mod errors;

pub use errors::ParseHandleError;
use hickory_proto::{
    ProtoError,
    rr::{IntoName, Name},
};
use regex::Regex;
use serde::{
    de::{self, Deserialize, Deserializer, Visitor},
    ser::{Serialize, Serializer},
};
use smol_str::{SmolStr, StrExt};
use std::{borrow::Borrow, fmt, net::IpAddr, str::FromStr, sync::LazyLock};

pub const INVALID_HANDLE: Handle = Handle(SmolStr::new_inline("handle.invalid"));
pub static REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$")
        .expect("invalid syntax")
});

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Handle(SmolStr);

impl Handle {
    #[inline(always)]
    pub fn new(handle: impl AsRef<str>) -> Result<Self, ParseHandleError> {
        handle.as_ref().parse()
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[inline(always)]
    pub fn is_invalid(&self) -> bool {
        self.0.ends_with(".invalid")
    }

    #[inline(always)]
    pub fn is_test(&self) -> bool {
        self.0.ends_with(".test")
    }
}

impl AsRef<[u8]> for Handle {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        AsRef::as_ref(&self.0)
    }
}

impl AsRef<str> for Handle {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        AsRef::as_ref(&self.0)
    }
}

impl Borrow<str> for Handle {
    #[inline(always)]
    fn borrow(&self) -> &str {
        Borrow::borrow(&self.0)
    }
}

impl<'de> Deserialize<'de> for Handle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HandleVisitor;
        impl<'de> Visitor<'de> for HandleVisitor {
            type Value = Handle;

            #[inline]
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "expecting a valid string representation of a handle"
                )
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

        deserializer.deserialize_newtype_struct("Handle", HandleVisitor)
    }
}

impl FromStr for Handle {
    type Err = ParseHandleError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if REGEX.is_match(raw) {
            let handle = raw.to_lowercase_smolstr();
            let tld = &handle[raw.rfind('.').unwrap_or_default() + 1..];

            match tld {
                "alt" | "arpa" | "example" | "internal" | "invalid" | "local" | "localhost"
                | "onion" => Err(Self::Err::InvalidTopLevelDomain),
                _ => Ok(Self(handle.into())),
            }
        } else {
            Err(Self::Err::InvalidSyntax)
        }
    }
}

impl IntoName for Handle {
    #[inline]
    fn into_name(self) -> Result<Name, ProtoError> {
        Name::from_labels(["_atproto", self.0.as_str()])
    }

    #[inline]
    fn to_ip(&self) -> Option<IpAddr> {
        IpAddr::from_str(&format!("_atproto.{}", self.0)).ok()
    }
}

impl IntoName for &Handle {
    #[inline]
    fn into_name(self) -> Result<Name, ProtoError> {
        Name::from_labels(["_atproto", self.0.as_str()])
    }

    #[inline]
    fn to_ip(&self) -> Option<IpAddr> {
        IpAddr::from_str(&format!("_atproto.{}", self.0)).ok()
    }
}

impl Serialize for Handle {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("Handle", &self.0)
    }
}

impl fmt::Display for Handle {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
