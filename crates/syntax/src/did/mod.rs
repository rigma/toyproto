mod errors;
pub mod methods;

pub use errors::ParseDidError;
pub use methods::Method;
use methods::Web;
use regex::Regex;
use serde::{
    de::{self, Deserialize, Visitor},
    ser::{Serialize, Serializer},
};
use smol_str::SmolStr;
use std::{borrow::Borrow, fmt, marker::PhantomData, str::FromStr, sync::LazyLock};

const MAX_LENGTH: usize = 2048;
static REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^did:[a-z]+:[a-zA-Z0-9._:%-]*[a-zA-Z0-9._-]$").expect("invalid syntax")
});

#[derive(Clone, Debug, Hash, Ord, PartialOrd)]
pub struct Did<M: Method> {
    pub identifier: SmolStr,
    method: PhantomData<M>,
}

impl<M> Did<M>
where
    M: Method,
{
    #[inline]
    pub fn new(identifier: impl Into<SmolStr>) -> Result<Self, ParseDidError> {
        let identifier = identifier.into();
        M::validate_identifier(&identifier)?;

        Ok(Self {
            identifier: identifier.into(),
            method: PhantomData,
        })
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        &self.identifier
    }
}

impl Did<Web> {
    #[inline]
    pub fn is_localhost(&self) -> bool {
        self.identifier.starts_with("localhost")
    }
}

impl<M> AsRef<[u8]> for Did<M>
where
    M: Method,
{
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        AsRef::as_ref(&self.identifier)
    }
}

impl<M> AsRef<str> for Did<M>
where
    M: Method,
{
    #[inline(always)]
    fn as_ref(&self) -> &str {
        AsRef::as_ref(&self.identifier)
    }
}

impl<M> Borrow<str> for Did<M>
where
    M: Method,
{
    #[inline(always)]
    fn borrow(&self) -> &str {
        Borrow::borrow(&self.identifier)
    }
}

impl<'de, M> Deserialize<'de> for Did<M>
where
    M: Method,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct DidVisitor<M: Method>(PhantomData<M>);
        impl<'de, M> Visitor<'de> for DidVisitor<M>
        where
            M: Method,
        {
            type Value = Did<M>;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "expecting a valid DID string using method '{}'",
                    M::name()
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
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_string(self)
            }
        }

        deserializer.deserialize_string(DidVisitor(PhantomData))
    }
}

impl<M: Method> Eq for Did<M> {}
impl<M> PartialEq for Did<M>
where
    M: Method,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl<M> FromStr for Did<M>
where
    M: Method,
{
    type Err = ParseDidError;

    #[inline]
    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            identifier: M::parse(raw)?,
            method: PhantomData,
        })
    }
}

impl<M> Serialize for Did<M>
where
    M: Method,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("Did", &format!("{self}"))
    }
}

impl<M> fmt::Display for Did<M>
where
    M: Method,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "did:{}:{}", M::name(), self.identifier)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Did, ParseDidError,
        methods::{Plc, Web},
    };
    use serde_test::{Token, assert_tokens};

    #[test]
    fn it_raises_errors_on_invalid_plc() {
        assert_eq!("did".parse::<Did<Plc>>(), Err(ParseDidError::InvalidSyntax));
        assert_eq!(
            "didmethodval".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "method:did:val".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "didmethod:val".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:methodval".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            ":did:method:val".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did.method.val".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "DID:method:val".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:METHOD:val".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:m123:val".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val:".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val%".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val/two".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val?two".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val#two".parse::<Did<Plc>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv".parse::<Did<Plc>>(),
            Err(ParseDidError::TooLong)
        );
        assert_eq!(
            "did:onion:2gzyxa5ihm7nsggfxnu52rck2vv4rvmdlkiu3zzui5du4xyclen53wid"
                .parse::<Did<Plc>>(),
            Err(ParseDidError::BadMethod("onion".into()))
        );
        assert_eq!(
            "did:example:123456789abcdefghi".parse::<Did<Plc>>(),
            Err(ParseDidError::BadMethod("example".into()))
        );
        assert_eq!(
            "did:web:example.com".parse::<Did<Plc>>(),
            Err(ParseDidError::BadMethod("web".into()))
        );
        assert_eq!(
            "did:web:localhost%3A1234".parse::<Did<Plc>>(),
            Err(ParseDidError::BadMethod("web".into()))
        );
        assert_eq!(
            "did:key:zQ3shZc2QzApp2oymGvQbzP8eKheVshBHbU4ZYjeXqwSKEn6No".parse::<Did<Plc>>(),
            Err(ParseDidError::BadMethod("key".into()))
        );
        assert_eq!(
            "did:ethr:0xb9c5714089478a327f09197987f16f9e5d936e8a".parse::<Did<Plc>>(),
            Err(ParseDidError::BadMethod("ethr".into()))
        );
        assert_eq!(
            "did:plc:7iza6de2dwap2sbkpav7c6c6666".parse::<Did<Plc>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:plc:8iza6de2dwap2sbkpav9c0c1".parse::<Did<Plc>>(),
            Err(ParseDidError::Unprocessable)
        );
    }

    #[test]
    fn it_raises_errors_on_invalid_web() {
        assert_eq!("did".parse::<Did<Web>>(), Err(ParseDidError::InvalidSyntax));
        assert_eq!(
            "didmethodval".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "method:did:val".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "didmethod:val".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:methodval".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            ":did:method:val".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did.method.val".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "DID:method:val".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:METHOD:val".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:m123:val".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val:".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val%".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val/two".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val?two".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:val#two".parse::<Did<Web>>(),
            Err(ParseDidError::InvalidSyntax)
        );
        assert_eq!(
            "did:method:vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv".parse::<Did<Web>>(),
            Err(ParseDidError::TooLong)
        );
        assert_eq!(
            "did:onion:2gzyxa5ihm7nsggfxnu52rck2vv4rvmdlkiu3zzui5du4xyclen53wid"
                .parse::<Did<Web>>(),
            Err(ParseDidError::BadMethod("onion".into()))
        );
        assert_eq!(
            "did:example:123456789abcdefghi".parse::<Did<Web>>(),
            Err(ParseDidError::BadMethod("example".into()))
        );
        assert_eq!(
            "did:plc:7iza6de2dwap2sbkpav7c6c6".parse::<Did<Web>>(),
            Err(ParseDidError::BadMethod("plc".into()))
        );
        assert_eq!(
            "did:key:zQ3shZc2QzApp2oymGvQbzP8eKheVshBHbU4ZYjeXqwSKEn6N".parse::<Did<Web>>(),
            Err(ParseDidError::BadMethod("key".into()))
        );
        assert_eq!(
            "did:ethr:0xb9c5714089478a327f09197987f16f9e5d936e8a".parse::<Did<Web>>(),
            Err(ParseDidError::BadMethod("ethr".into()))
        );
        assert_eq!(
            "did:web:example.alt".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:example.arpa".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:example.example".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:example.internal".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:example.invalid".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:example.local".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:example.localhost".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:example.onion".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:localhost%3A".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:localhost%3A123456".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
        assert_eq!(
            "did:web:localhost%3A65536".parse::<Did<Web>>(),
            Err(ParseDidError::Unprocessable)
        );
    }

    #[test]
    fn it_serializes_and_deserializes_plc_method() {
        let did: Did<Plc> = Did::new("7iza6de2dwap2sbkpav7c6c6").unwrap();

        assert_tokens(
            &did,
            &[
                Token::NewtypeStruct { name: "Did" },
                Token::String("did:plc:7iza6de2dwap2sbkpav7c6c6"),
            ],
        );
    }

    #[test]
    fn it_serializes_and_deserializes_web_method() {
        let did: Did<Web> = Did::new("example.com").unwrap();

        assert_tokens(
            &did,
            &[
                Token::NewtypeStruct { name: "Did" },
                Token::String("did:web:example.com"),
            ],
        );
    }
}
