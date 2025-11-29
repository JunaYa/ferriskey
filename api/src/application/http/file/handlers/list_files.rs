use axum::{
    Extension,
    extract::{Path, Query, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    storage::{
        entities::StoredObject,
        services::FileService,
        value_objects::{OffsetLimit, Paginated, StoredObjectFilter},
    },
};

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

use crate::application::http::file::validators::ListFilesQuery;

#[utoipa::path(
    get,
    path = "",
    tag = "file",
    summary = "List files in a realm",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("offset" = Option<i64>, Query, description = "Pagination offset (default: 0)"),
        ("limit" = Option<i64>, Query, description = "Pagination limit (default: 20, max: 100)"),
        ("mime_type" = Option<String>, Query, description = "Filter by MIME type"),
        ("uploaded_by" = Option<String>, Query, description = "Filter by uploader user ID"),
    ),
    responses(
        (status = 200, description = "Files listed successfully", body = Paginated<StoredObject>),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn list_files(
    Path(_realm_name): Path<String>,
    Query(query): Query<ListFilesQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Paginated<StoredObject>>, ApiError> {
    let filter = StoredObjectFilter {
        mime_type: query.mime_type,
        uploaded_by: query.uploaded_by,
        created_before: query.created_before,
        created_after: query.created_after,
    };

    let pagination = OffsetLimit::new(
        query.offset.unwrap_or(0),
        query.limit.unwrap_or(20).min(100),
    );

    let result = state
        .service
        .list_files(identity, filter, pagination)
        .await?;

    Ok(Response::OK(result))
}
