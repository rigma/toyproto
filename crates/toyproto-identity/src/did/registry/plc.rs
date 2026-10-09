use super::{BuildRegistryError, FetchDocumentError, RegistryClient};
use crate::did::Document;
use reqwest::{
    Client,
    header::{self, HeaderMap, HeaderValue},
};
use std::time::Duration;
use toyproto_syntax::did::{Did, methods::Method};

const BASE_URL: &str = "https://plc.directory";

#[derive(Clone, Debug)]
pub(crate) struct Plc {
    client: Client,
}

impl RegistryClient for Plc {
    #[inline]
    fn new() -> Result<Self, BuildRegistryError> {
        let client = Client::builder().timeout(Duration::from_secs(10)).build()?;

        Ok(Self { client })
    }

    #[inline]
    fn with_user_agent(user_agent: impl AsRef<str>) -> Result<Self, BuildRegistryError> {
        let client = Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(header::USER_AGENT, user_agent.as_ref().parse()?);

                headers
            })
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(Self { client })
    }

    async fn fetch_document<M>(&self, did: &Did<M>) -> Result<Document<M>, FetchDocumentError>
    where
        M: Method + Sync,
    {
        let response = self
            .client
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
}
