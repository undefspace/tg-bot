pub mod services;

use http::uri::{InvalidUri, PathAndQuery};
use http::Method;
use serde::{Deserialize, Serialize};

pub trait Endpoint: Serialize {
    const METHOD: Method;
    type Output: for<'de> Deserialize<'de>;
    fn path_and_query(&self) -> Result<PathAndQuery, InvalidUri>;
}
