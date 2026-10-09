mod builder;
mod errors;

use crate::handle::Handle;
use builder::NsidBuilder;
pub use errors::{BuildNsidError, ParseNsidError};
use hickory_proto::{
    ProtoError,
    rr::{IntoName, Name},
};
use regex::Regex;
use smol_str::{SmolStr, ToSmolStr};
use std::{fmt, net::IpAddr, str::FromStr, sync::LazyLock};

static REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+(\.[a-zA-Z]([a-zA-Z0-9]{0,62})?)$")
        .expect("invalid syntax")
});

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Nsid {
    pub authority: Handle,
    pub name: SmolStr,
}

impl Nsid {
    #[inline]
    #[must_use]
    pub fn new() -> NsidBuilder {
        NsidBuilder::new()
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.to_string()
    }

    #[inline]
    pub fn to_string(&self) -> String {
        let domain = self
            .authority
            .as_str()
            .split('.')
            .rev()
            .collect::<Vec<_>>()
            .join(".");

        format!("{domain}.{}", self.name)
    }
}

#[cfg(test)]
impl From<&str> for Nsid {
    #[inline]
    fn from(value: &str) -> Self {
        value.parse().expect("invalid nsid syntax")
    }
}

impl FromStr for Nsid {
    type Err = ParseNsidError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        if raw.len() == 0 {
            return Err(Self::Err::EmptyString);
        } else if raw.len() > 317 {
            return Err(Self::Err::TooLong);
        } else if !REGEX.is_match(raw) {
            return Err(Self::Err::InvalidSyntax);
        }

        let parts: Vec<&str> = raw.split('.').rev().collect();
        if parts.len() < 3 {
            return Err(Self::Err::InvalidSyntax);
        }

        let authority = parts[1..].join(".");
        let name = parts.first().unwrap();

        Ok(Self {
            authority: authority.parse()?,
            name: name.to_smolstr(),
        })
    }
}

impl IntoName for &Nsid {
    #[inline]
    fn into_name(self) -> Result<Name, ProtoError> {
        Name::from_labels(self.authority.labels_with_prefix("_lexicon"))
    }

    #[inline]
    fn to_ip(&self) -> Option<std::net::IpAddr> {
        IpAddr::from_str(&self.authority.labels_with_prefix("_lexicon").join(".")).ok()
    }
}

impl IntoName for Nsid {
    #[inline]
    fn into_name(self) -> Result<Name, ProtoError> {
        Name::from_labels(self.authority.labels_with_prefix("_lexicon"))
    }

    #[inline]
    fn to_ip(&self) -> Option<IpAddr> {
        IpAddr::from_str(&self.authority.labels_with_prefix("_lexicon").join(".")).ok()
    }
}

impl TryFrom<Box<[u8]>> for Nsid {
    type Error = ParseNsidError;

    #[inline]
    fn try_from(value: Box<[u8]>) -> Result<Self, Self::Error> {
        String::from_utf8_lossy(&value).parse()
    }
}

impl TryFrom<&[u8]> for Nsid {
    type Error = ParseNsidError;

    #[inline]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        String::from_utf8_lossy(value).parse()
    }
}

impl fmt::Display for Nsid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.authority.as_str(), self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_valid_nsid() {
        assert_eq!(
            "com.ooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo.foo"
                .parse::<Nsid>(),
            Ok(Nsid {
                authority: "ooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo.com"
                    .into(),
                name: "foo".into(),
            })
        );
        assert_eq!(
            "com.example.ooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo"
                .parse::<Nsid>(),
            Ok(Nsid {
                authority: "example.com".into(),
                name: "ooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo".into()
            })
        );
        assert_eq!(
            "com.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.foo".parse::<Nsid>(),
            Ok(Nsid {
                authority: "middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.com".into(),
                name: "foo".into(),
            })
        );
        assert_eq!(
            "com.example.fooBar".parse::<Nsid>(),
            Ok(Nsid {
                authority: "example.com".into(),
                name: "fooBar".into()
            })
        );
        assert_eq!(
            "com.example.fooBarV2".parse::<Nsid>(),
            Ok(Nsid {
                authority: "example.com".into(),
                name: "fooBarV2".into(),
            })
        );
        assert_eq!(
            "net.users.bob.ping".parse::<Nsid>(),
            Ok(Nsid {
                authority: "bob.users.net".into(),
                name: "ping".into(),
            })
        );
        assert_eq!(
            "a.b.c".parse::<Nsid>(),
            Ok(Nsid {
                authority: "b.a".into(),
                name: "c".into(),
            })
        );
        assert_eq!(
            "m.xn--masekowski-d0b.pl".parse::<Nsid>(),
            Ok(Nsid {
                authority: "xn--masekowski-d0b.m".into(),
                name: "pl".into(),
            })
        );
        assert_eq!(
            "one.two.three".parse::<Nsid>(),
            Ok(Nsid {
                authority: "two.one".into(),
                name: "three".into(),
            })
        );
        assert_eq!(
            "one.two.three.four-and.FiVe".parse::<Nsid>(),
            Ok(Nsid {
                authority: "four-and.three.two.one".into(),
                name: "FiVe".into(),
            })
        );
        assert_eq!(
            "one.2.three".parse::<Nsid>(),
            Ok(Nsid {
                authority: "2.one".into(),
                name: "three".into(),
            })
        );
        assert_eq!(
            "a-0.b-1.c".parse::<Nsid>(),
            Ok(Nsid {
                authority: "b-1.a-0".into(),
                name: "c".into(),
            })
        );
        assert_eq!(
            "a0.b1.cc".parse::<Nsid>(),
            Ok(Nsid {
                authority: "b1.a0".into(),
                name: "cc".into(),
            })
        );
        assert_eq!(
            "cn.8.lex.stuff".parse::<Nsid>(),
            Ok(Nsid {
                authority: "lex.8.cn".into(),
                name: "stuff".into(),
            })
        );
        assert_eq!(
            "test.12345.record".parse::<Nsid>(),
            Ok(Nsid {
                authority: "12345.test".into(),
                name: "record".into(),
            })
        );
        assert_eq!(
            "a01.thing.record".parse::<Nsid>(),
            Ok(Nsid {
                authority: "thing.a01".into(),
                name: "record".into(),
            })
        );
        assert_eq!(
            "a.0.c".parse::<Nsid>(),
            Ok(Nsid {
                authority: "0.a".into(),
                name: "c".into(),
            })
        );
        assert_eq!(
            "xn--fiqs8s.xn--fiqa61au8b7zsevnm8ak20mc4a87e.record.two".parse::<Nsid>(),
            Ok(Nsid {
                authority: "record.xn--fiqa61au8b7zsevnm8ak20mc4a87e.xn--fiqs8s".into(),
                name: "two".into(),
            })
        );
        assert_eq!(
            "a0.b1.c3".parse::<Nsid>(),
            Ok(Nsid {
                authority: "b1.a0".into(),
                name: "c3".into(),
            })
        );
        assert_eq!(
            "com.example.f00".parse::<Nsid>(),
            Ok(Nsid {
                authority: "example.com".into(),
                name: "f00".into(),
            })
        );
        assert_eq!(
            "org.4chan.lex.getThing".parse::<Nsid>(),
            Ok(Nsid {
                authority: "lex.4chan.org".into(),
                name: "getThing".into(),
            })
        );
        assert_eq!(
            "cn.8.lex.stuff".parse::<Nsid>(),
            Ok(Nsid {
                authority: "lex.8.cn".into(),
                name: "stuff".into(),
            })
        );
    }

    #[test]
    fn it_raises_errors_when_parsing_invalid_nsid() {
        assert_eq!("".parse::<Nsid>(), Err(ParseNsidError::EmptyString));
        assert_eq!(
            "com.oooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo.foo"
                .parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.example.oooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo"
                .parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.middle.foo".parse::<Nsid>(),
            Err(ParseNsidError::TooLong)
        );
        assert_eq!(
            "com.example.foo.*".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.example.foo.blah*".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.example.foo.*blah".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.exa💩ple.thing".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "a-0.b-1.c-3".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "a-0.b-1.c-o".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "1.0.0.127.record".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "0two.example.foo".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "example.com".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.example".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!("a.".parse::<Nsid>(), Err(ParseNsidError::InvalidSyntax));
        assert_eq!(
            ".one.two.three".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "one.two.three ".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "one.two..three".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "one .two.three".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            " one.two.three".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.exa💩ple.thing".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.atproto.feed.p@st".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.atproto.feed.p_st".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.atproto.feed.p*st".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.atproto.feed.po#t".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.atproto.feed.p!ot".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.example-.foo".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "com.example.fooBar.2".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "onion.expyuzz4wqqyqhjn.spec.getThing".parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
        assert_eq!(
            "onion.g2zyxa5ihm7nsggfxnu52rck2vv4rvmdlkiu3zzui5du4xyclen53wid.lex.deleteThing"
                .parse::<Nsid>(),
            Err(ParseNsidError::InvalidSyntax)
        );
    }
}
