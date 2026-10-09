use super::{
    Directory,
    builder::{DirectoryBuilder, DirectoryFromParts},
    errors::LookupError,
};
use crate::{
    Identity,
    did::{Document, Resolver as DidResolver, errors::ResolveDidError, registry::Registry},
    dns::parse_txt_lookup,
    handle::{Resolver as HandleResolver, errors::ResolveHandleError},
    nsid::{Resolver as NsidResolver, errors::ResolveNsidError},
};
use hickory_resolver::TokioResolver as DnsResolver;
use reqwest::{Client, header};
use toyproto_syntax::{
    did::{
        Did,
        methods::{Method, Plc, Universal, Web},
    },
    handle::Handle,
    nsid::Nsid,
};

#[derive(Clone, Debug)]
pub struct DefaultDirectory {
    http_client: Client,
    did_registry: Registry,
    dns_resolver: DnsResolver,
}

impl DefaultDirectory {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        DirectoryBuilder::<Self>::new()
            .build()
            .expect("unable to instanciate DID directory")
    }

    #[inline]
    #[must_use]
    pub fn builder() -> DirectoryBuilder<Self> {
        DirectoryBuilder::new()
    }
}

impl DefaultDirectory {
    async fn resolve_handle_dns<M>(&self, handle: &Handle) -> Result<Did<M>, ResolveHandleError>
    where
        M: Method,
    {
        let lookup = self.dns_resolver.txt_lookup(handle).await?;
        let did = parse_txt_lookup(lookup);

        did.ok_or(ResolveHandleError::NotFound)
    }

    async fn resolve_handle_well_known<M>(
        &self,
        handle: &Handle,
    ) -> Result<Did<M>, ResolveHandleError>
    where
        M: Method,
    {
        let response = self
            .http_client
            .get(format!("https://{handle}/.well-known/atproto-did"))
            .send()
            .await?
            .error_for_status()?;
        let did = response.text().await?.trim().parse()?;

        Ok(did)
    }
}

impl<M> Directory<M> for DefaultDirectory
where
    Self: DidResolver<M> + HandleResolver<M>,
    M: Method + Send + Sync,
{
    async fn lookup_did(&self, did: &Did<M>) -> Result<Identity<M>, LookupError> {
        let mut identity: Identity<M> = self.resolve_did(did).await?.into();
        // TODO: add option to skip declared handle verification

        if let Some(declared_handle) = identity.declared_handle() {
            let resolved_did = self.resolve_handle(&declared_handle).await;
            if let Ok(resolved_did) = resolved_did
                && resolved_did == did
            {
                identity.handle = declared_handle;
            }
        }

        Ok(identity)
    }

    async fn lookup_handle(&self, handle: &Handle) -> Result<Identity<M>, LookupError> {
        let did = self.resolve_handle(handle).await?;
        let mut identity: Identity<M> = self.resolve_did(&did).await?.into();

        let declared_handle = identity
            .declared_handle()
            .ok_or(LookupError::UncheckedHandle)?;

        if declared_handle == handle {
            identity.handle = declared_handle;
            Ok(identity)
        } else {
            Err(LookupError::HandleMismatch {
                declared: declared_handle.to_string(),
                given: handle.to_string(),
            })
        }
    }
}

impl DidResolver<Plc> for DefaultDirectory {
    async fn resolve_did(&self, did: &Did<Plc>) -> Result<Document<Plc>, ResolveDidError> {
        let document = self.did_registry.resolve(did).await?;

        Ok(document)
    }
}

impl DidResolver<Universal> for DefaultDirectory
where
    Self: DidResolver<Plc> + DidResolver<Web>,
{
    async fn resolve_did(&self, did: &Did<Universal>) -> Result<Document, ResolveDidError> {
        if did.starts_with("did:plc:") {
            let plc: Did<Plc> = did
                .to_method()
                .map_err(|_| ResolveDidError::ResolutionFailed)?;
            let document = self.resolve_did(&plc).await?;

            Ok(Document {
                id: did.clone(),
                also_known_as: document.also_known_as,
                verification_method: document.verification_method,
                service: document.service,
            })
        } else if did.starts_with("did:web:") {
            let web: Did<Web> = did
                .to_method()
                .map_err(|_| ResolveDidError::ResolutionFailed)?;
            let document = self.resolve_did(&web).await?;

            Ok(Document {
                id: did.clone(),
                also_known_as: document.also_known_as,
                verification_method: document.verification_method,
                service: document.service,
            })
        } else {
            Err(ResolveDidError::ResolutionFailed)
        }
    }
}

impl DidResolver<Web> for DefaultDirectory {
    async fn resolve_did(&self, did: &Did<Web>) -> Result<Document<Web>, ResolveDidError> {
        let response = self
            .http_client
            .get(if did.is_localhost() {
                format!(
                    "http://{}/.well-known/did.json",
                    did.identifier.replace("%3A", ":")
                )
            } else {
                format!("https://{}/.well-known/did.json", did.identifier)
            })
            .header(header::ACCEPT, "application/did+ld+json, application/json")
            .send()
            .await?
            .error_for_status()?;
        let document = response.json::<Document<_>>().await?;

        Ok(document)
    }
}

impl<M> HandleResolver<M> for DefaultDirectory
where
    M: Method + Send,
{
    async fn resolve_handle(&self, handle: &Handle) -> Result<Did<M>, ResolveHandleError> {
        let did = self.resolve_handle_dns(handle).await;
        if let Err(dns_error) = did {
            self.resolve_handle_well_known(handle)
                .await
                .or_else(|http_error| {
                    Err(if dns_error != ResolveHandleError::NotFound {
                        dns_error
                    } else if http_error != ResolveHandleError::NotFound {
                        http_error
                    } else {
                        dns_error
                    })
                })
        } else {
            did
        }
    }
}

impl NsidResolver for DefaultDirectory {
    async fn resolve_nsid<M>(&self, nsid: &Nsid) -> Result<Did<M>, ResolveNsidError>
    where
        M: Method,
    {
        let lookup = self.dns_resolver.txt_lookup(nsid).await?;
        let did = parse_txt_lookup(lookup);

        did.ok_or(ResolveNsidError::ResolutionFailed)
    }
}

impl DirectoryFromParts for DefaultDirectory {
    type Output = Self;

    #[inline]
    fn from_parts(
        http_client: Client,
        did_registry: Registry,
        dns_resolver: DnsResolver,
    ) -> Self::Output {
        Self {
            http_client,
            did_registry,
            dns_resolver,
        }
    }
}
