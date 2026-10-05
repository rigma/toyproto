use crate::did::Document;
use std::{collections::BTreeMap, fmt};
use toyproto_syntax::{
    did::{
        Did,
        methods::{Method, Universal},
    },
    handle::{Handle, INVALID_HANDLE},
};

#[derive(Clone, Debug)]
pub struct Identity<M: Method = Universal> {
    pub did: Did<M>,

    pub handle: Handle,

    pub also_known_as: Vec<String>,

    pub services: BTreeMap<String, ServiceEndpoint>,

    pub keys: BTreeMap<String, VerificationMethod>,
}

impl<M> Identity<M>
where
    M: Method,
{
    #[inline]
    pub fn declared_handle(&self) -> Option<Handle> {
        self.also_known_as.iter().find_map(|id| {
            if id.starts_with("at://") && id.len() > 5 {
                id[5..].parse().ok()
            } else {
                None
            }
        })
    }
}

impl<M> From<Document<M>> for Identity<M>
where
    M: Method,
{
    fn from(value: Document<M>) -> Self {
        let keys = BTreeMap::from_iter(
            value
                .verification_method
                .into_iter()
                .filter(|method| value.id == method.controller)
                .map(|method| {
                    method.id.split_once('#').and_then(|(_, fragment)| {
                        Some((
                            fragment.into(),
                            VerificationMethod {
                                ty: method.ty,
                                public_key_multibase: method.public_key_multibase,
                            },
                        ))
                    })
                })
                .flatten(),
        );

        Self {
            did: value.id,
            handle: INVALID_HANDLE,
            also_known_as: value.also_known_as,
            services: BTreeMap::from_iter(
                value
                    .service
                    .into_iter()
                    .map(|service| {
                        service.id.split_once('#').and_then(|(_, fragment)| {
                            Some((
                                fragment.into(),
                                ServiceEndpoint {
                                    ty: service.ty,
                                    url: service.service_endpoint,
                                },
                            ))
                        })
                    })
                    .flatten(),
            ),
            keys,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServiceEndpoint {
    pub ty: String,

    pub url: String,
}

impl fmt::Display for ServiceEndpoint {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServiceEndpoint(type = {}, url = {})", self.ty, self.url)
    }
}

#[derive(Clone, Debug)]
pub struct VerificationMethod {
    pub ty: String,

    pub public_key_multibase: String,
}

impl fmt::Display for VerificationMethod {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VerificationMethod(type = {}, pk_multibase = {})",
            self.ty, self.public_key_multibase
        )
    }
}
