use std::fmt::Debug;

use http::{
    uri::{self, Authority, InvalidUri, InvalidUriParts, Scheme},
    Uri,
};
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use thiserror::Error;
use tracing::{debug, instrument};

use super::endpoints::Endpoint;
use super::http_utils::Downgrade;

static DEFAULT_AUTHORITY: Lazy<Authority> = Lazy::new(|| Authority::from_static("localhost:8123"));

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum NewClientError {
    #[error("this token is invalid as a header value")]
    InvalidToken,
    #[error("could not create a reqwest::Client: {0}")]
    ReqwestError(#[from] reqwest::Error),
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum RequestError {
    #[error("invalid endpoint URI: {0}")]
    InvalidEndpoint(#[from] InvalidUri),
    #[error("could not build the request url: {0}")]
    InvalidUri(#[from] InvalidUriParts),
    #[error("request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("could not parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub struct Client {
    pub authority: Authority,
    client: reqwest::Client,
}

impl Client {
    pub fn with_builder(
        builder: reqwest::ClientBuilder,
        token: &str,
    ) -> Result<Self, NewClientError> {
        {
            let mut headers = HeaderMap::with_capacity(1);
            let Ok(mut bearer_auth) = format!("Bearer {token}").parse::<HeaderValue>() else {
                return Err(NewClientError::InvalidToken);
            };
            bearer_auth.set_sensitive(true);
            headers.insert(AUTHORIZATION, bearer_auth);

            let client = builder.default_headers(headers).build()?;

            Ok(Self {
                authority: DEFAULT_AUTHORITY.to_owned(),
                client,
            })
        }
    }

    pub fn new(token: &str) -> Result<Self, NewClientError> {
        Self::with_builder(reqwest::ClientBuilder::new(), token)
    }

    #[instrument(skip(request))]
    pub async fn execute<R>(&self, request: R) -> Result<R::Output, RequestError>
    where
        R: Endpoint + Send,
    {
        let parts = {
            let mut parts = uri::Parts::default();
            parts.scheme = Some(Scheme::HTTP);
            parts.authority = Some(self.authority.clone());
            parts.path_and_query = Some(request.path_and_query()?);
            parts
        };
        let request = self
            .client
            .request(R::METHOD.downgrade(), Uri::from_parts(parts)?.to_string())
            .json(&request)
            .build()?;
        debug!("Request: {request:?}");
        debug!(
            "Request body: {:?}",
            request
                .body()
                .and_then(|x| x.as_bytes())
                .map(bstr::ByteSlice::as_bstr)
        );
        let response = self.client.execute(request).await?;
        let err = response.error_for_status_ref().err();
        debug!("Response: {response:?}");
        let bytes = response.bytes().await?;
        debug!("Response body: {bytes:?}");
        if let Some(err) = err {
            return Err(err.into());
        }
        Ok(serde_json::from_slice(&bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use http::uri::Authority;

    use super::DEFAULT_AUTHORITY;

    #[test]
    fn valid_base() {
        let _: Authority = *DEFAULT_AUTHORITY;
    }
}
