use crate::{
    did::{Did, Method},
    handle::Handle,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Identifier<M: Method> {
    Did(Did<M>),

    Handle(Handle),
}

impl<M> From<Did<M>> for Identifier<M>
where
    M: Method,
{
    #[inline(always)]
    fn from(value: Did<M>) -> Self {
        Self::Did(value)
    }
}

impl<M> From<Handle> for Identifier<M>
where
    M: Method,
{
    #[inline(always)]
    fn from(value: Handle) -> Self {
        Self::Handle(value)
    }
}
