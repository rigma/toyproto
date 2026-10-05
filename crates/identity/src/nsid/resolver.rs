use super::errors::ResolveNsidError;
use toyproto_syntax::{
    did::{Did, methods::Method},
    nsid::Nsid,
};

pub trait Resolver {
    fn resolve_nsid<M>(
        &self,
        nsid: &Nsid,
    ) -> impl Future<Output = Result<Did<M>, ResolveNsidError>>
    where
        M: Method;
}
