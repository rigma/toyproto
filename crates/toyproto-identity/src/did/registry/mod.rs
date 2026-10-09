mod errors;
#[cfg(feature = "slingshot")]
mod slingshot;

use std::time::Duration;

use crate::did::Document;
pub use errors::ResolveError;
use reqwest::{
    Client,
    header::{self, HeaderMap, HeaderValue},
};
use toyproto_syntax::did::{Did, methods::Method};

#[cfg(not(feature = "slingshot"))]
const BASE_URL: &str = "https://plc.directory";
#[cfg(feature = "slingshot")]
const BASE_URL: &str =
    "https://slingshot.microcosm.blue/xrpc/blue.microcosm.identity.resolveMiniDoc";
const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug)]
pub struct Registry(Client);

impl Registry {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    #[must_use]
    pub fn with_user_agent(user_agent: impl AsRef<str>) -> Self {
        let client = Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::USER_AGENT,
                    user_agent
                        .as_ref()
                        .parse()
                        .expect("provided user agent is not a valid HTTP header value"),
                );

                headers
            })
            .timeout(TIMEOUT)
            .build()
            .expect("unable to instanciate DID registry client");

        Self(client)
    }

    #[cfg(not(feature = "slingshot"))]
    #[must_use]
    pub async fn resolve<M>(&self, did: &Did<M>) -> Result<Document<M>, ResolveError>
    where
        M: Method,
    {
        let response = self
            .0
            .get(format!("{BASE_URL}/{did}"))
            .header(
                header::ACCEPT,
                HeaderValue::from_static("application/did+ld+json, application/json"),
            )
            .send()
            .await?;
        let document = response.json().await?;

        Ok(document)
    }

    #[cfg(feature = "slingshot")]
    #[must_use]
    pub async fn resolve<M>(&self, did: &Did<M>) -> Result<Document<M>, ResolveError>
    where
        M: Method,
    {
        let response = self
            .0
            .get(format!("{BASE_URL}?identifier={did}"))
            .header(
                header::ACCEPT,
                HeaderValue::from_static("application/json;charset=utf-8"),
            )
            .send()
            .await?;
        let document: slingshot::MiniDoc<M> = response.json().await?;

        Ok(document.into())
    }
}

impl Default for Registry {
    #[inline]
    fn default() -> Self {
        let client = Client::builder()
            .timeout(TIMEOUT)
            .build()
            .expect("unable to instanciate DID registry");

        Self(client)
    }
}
