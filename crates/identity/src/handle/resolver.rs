use super::errors::ResolveHandleError;
use toyproto_syntax::{
    did::{Did, methods::Method},
    handle::Handle,
};

pub trait Resolver<M: Method> {
    fn resolve_handle(
        &self,
        handle: &Handle,
    ) -> impl Future<Output = Result<Did<M>, ResolveHandleError>> + Send;
}
