use axum::{extract::FromRequestParts, http::request::Parts, response::Response};
use std::collections::HashMap;

use super::query_params::QueryParams;

/// Extractor for query parameters that supports filter, sort, and pagination
///
/// Usage:
/// ```rust
/// async fn handler(
///     QueryParamsExtractor(query_params): QueryParamsExtractor,
/// ) -> Result<Response, ApiError> {
///     // Use query_params.filter, query_params.sort, query_params.pagination
/// }
/// ```
#[derive(Debug, Clone)]
pub struct QueryParamsExtractor(pub QueryParams);

impl<S> FromRequestParts<S> for QueryParamsExtractor
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Parse query string into HashMap
        let query_string = parts.uri.query().unwrap_or("");
        let query_map: HashMap<String, String> =
            serde_urlencoded::from_str(query_string).unwrap_or_default();

        // Parse into QueryParams
        let query_params = QueryParams::from_query_map(&query_map);

        Ok(QueryParamsExtractor(query_params))
    }
}
