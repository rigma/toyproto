use super::{Document, errors::ResolveDidError};
use toyproto_syntax::did::{Did, methods::Method};

pub trait Resolver<M: Method> {
    fn resolve_did(
        &self,
        did: &Did<M>,
    ) -> impl Future<Output = Result<Document<M>, ResolveDidError>> + Send;
}
