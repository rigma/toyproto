use super::{BuildNsidError, Nsid};
use regex::Regex;
use smol_str::SmolStr;
use std::{marker::PhantomData, sync::LazyLock};

static NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9]{0,62}?$").expect("invalid syntax"));

pub trait State: crate::private::Sealed {}

pub struct NeedsAuthority;
impl State for NeedsAuthority {}
impl crate::private::Sealed for NeedsAuthority {}

pub struct NeedsName;
impl State for NeedsName {}
impl crate::private::Sealed for NeedsName {}

pub struct Ready;
impl State for Ready {}
impl crate::private::Sealed for Ready {}

#[derive(Clone, Debug)]
pub struct NsidBuilder<S: State = NeedsAuthority> {
    authority: Option<SmolStr>,
    name: Option<SmolStr>,
    state: PhantomData<S>,
}

impl NsidBuilder {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    #[must_use]
    pub fn authority(self, authority: impl AsRef<str>) -> NsidBuilder<NeedsAuthority> {
        NsidBuilder {
            authority: Some(authority.as_ref().into()),
            name: self.name,
            state: PhantomData,
        }
    }
}

impl NsidBuilder<NeedsName> {
    #[inline]
    #[must_use]
    pub fn name(self, name: impl AsRef<str>) -> NsidBuilder<Ready> {
        NsidBuilder {
            authority: self.authority,
            name: Some(name.as_ref().into()),
            state: PhantomData,
        }
    }
}

impl NsidBuilder<Ready> {
    #[inline]
    #[must_use]
    pub fn build(self) -> Result<Nsid, BuildNsidError> {
        let authority = self.authority.unwrap();
        let name = self.name.unwrap();

        if authority.len() > 253 {
            Err(BuildNsidError::InvalidDomain)
        } else if !NAME_REGEX.is_match(&name) {
            Err(BuildNsidError::InvalidName)
        } else {
            Ok(Nsid {
                authority: authority.parse()?,
                name,
            })
        }
    }
}

impl Default for NsidBuilder {
    #[inline]
    fn default() -> Self {
        Self {
            authority: None,
            name: None,
            state: PhantomData,
        }
    }
}
