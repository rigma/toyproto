use super::{Directory, errors::BuilderError};
use crate::did::registry::Registry;
use hickory_resolver::{Resolver, config::ResolverConfig, net::runtime::TokioRuntimeProvider};
use reqwest::{
    Client,
    header::{self, HeaderMap},
};
use std::{marker::PhantomData, time::Duration};

const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(1);
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(3);

pub trait DirectoryFromParts: Directory {
    type Output: Directory;

    fn from_parts(
        http_client: Client,
        did_registry: Registry,
        dns_resolver: Resolver<TokioRuntimeProvider>,
    ) -> Self::Output;
}

#[derive(Debug)]
pub struct DirectoryBuilder<D: DirectoryFromParts> {
    dns_resolver_config: Option<ResolverConfig>,
    idle_timeout: Option<Duration>,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    _directory: PhantomData<D>,
}

impl<D> DirectoryBuilder<D>
where
    D: DirectoryFromParts,
{
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    #[must_use]
    pub fn idle_timeout(self, secs: u64) -> Self {
        Self {
            idle_timeout: Some(Duration::from_secs(secs)),
            ..self
        }
    }

    #[inline]
    #[must_use]
    pub fn idle_timeout_millis(self, millis: u64) -> Self {
        Self {
            idle_timeout: Some(Duration::from_millis(millis)),
            ..self
        }
    }

    #[inline]
    #[must_use]
    pub fn resolver_config(self, config: ResolverConfig) -> Self {
        Self {
            dns_resolver_config: Some(config),
            ..self
        }
    }

    #[inline]
    #[must_use]
    pub fn timeout(self, secs: u64) -> Self {
        Self {
            timeout: Some(Duration::from_secs(secs)),
            ..self
        }
    }

    #[inline]
    #[must_use]
    pub fn timeout_millis(self, millis: u64) -> Self {
        Self {
            timeout: Some(Duration::from_millis(millis)),
            ..self
        }
    }

    #[inline]
    #[must_use]
    pub fn user_agent(self, user_agent: impl Into<String>) -> Self {
        Self {
            user_agent: Some(user_agent.into()),
            ..self
        }
    }

    #[inline]
    #[must_use]
    pub fn build(self) -> Result<D::Output, BuilderError> {
        let client = {
            let mut builder = Client::builder()
                .pool_idle_timeout(self.idle_timeout.unwrap_or(DEFAULT_IDLE_TIMEOUT))
                .timeout(self.timeout.unwrap_or(DEFAULT_TIMEOUT));

            if let Some(user_agent) = &self.user_agent {
                builder = builder.default_headers({
                    let mut headers = HeaderMap::new();
                    headers.insert(header::USER_AGENT, user_agent.parse()?);

                    headers
                });
            }

            builder.build()?
        };
        let registry = self.user_agent.map_or_else(Registry::new, |user_agent| {
            Registry::with_user_agent(user_agent)
        });
        let resolver = self
            .dns_resolver_config
            .map_or_else(Resolver::builder_tokio, |config| {
                Ok(Resolver::builder_with_config(
                    config,
                    TokioRuntimeProvider::new(),
                ))
            })?
            .build()?;

        Ok(D::from_parts(client, registry, resolver))
    }
}

impl<D> Default for DirectoryBuilder<D>
where
    D: DirectoryFromParts,
{
    fn default() -> Self {
        Self {
            dns_resolver_config: None,
            idle_timeout: None,
            timeout: None,
            user_agent: None,
            _directory: PhantomData,
        }
    }
}
