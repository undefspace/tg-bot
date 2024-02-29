use super::super::types::Service;
use http::{
    uri::{InvalidUri, PathAndQuery},
    Method,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Post<S>(pub S)
where
    S: Service;

impl<S> super::Endpoint for Post<S>
where
    S: Service,
{
    const METHOD: http::Method = Method::POST;

    type Output = S::Output;

    fn path_and_query(&self) -> Result<PathAndQuery, InvalidUri> {
        format!("/api/services/{}/{}", self.0.domain(), self.0.service()).parse()
    }
}
