mod builder;
mod default_directory;
pub mod errors;

use crate::{Identity, did::Resolver as DidResolver, handle::Resolver as HandleResolver};
pub use default_directory::DefaultDirectory;
use errors::LookupError;
use toyproto_syntax::{
    Identifier,
    did::{
        Did,
        methods::{Method, Universal},
    },
    handle::Handle,
};

pub trait Directory<M: Method = Universal>: DidResolver<M> + HandleResolver<M> {
    fn lookup_did(
        &self,
        did: &Did<M>,
    ) -> impl Future<Output = Result<Identity<M>, LookupError>> + Send
    where
        M: Method;

    fn lookup_handle(
        &self,
        handle: &Handle,
    ) -> impl Future<Output = Result<Identity<M>, LookupError>> + Send;

    fn lookup(
        &self,
        identitier: &Identifier<M>,
    ) -> impl Future<Output = Result<Identity<M>, LookupError>> + Send
    where
        Self: Sync,
        M: Method + Sync,
    {
        async {
            match identitier {
                Identifier::Did(did) => self.lookup_did(did).await,
                Identifier::Handle(handle) => self.lookup_handle(handle).await,
            }
        }
    }

    #[allow(unused)]
    fn purge(&self, identifier: &Identifier<M>)
    where
        M: Method,
    {
        unimplemented!()
    }
}
