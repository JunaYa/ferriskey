use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    storage::{entities::StoredObject, services::FileService},
};
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[utoipa::path(
    post,
    path = "/{file_id}/complete",
    tag = "file",
    summary = "Complete a file upload",
    responses(
        (status = 200, description = "Upload completed successfully", body = StoredObject),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "File not found")
    )
)]
pub async fn complete_upload(
    Path((_realm_name, file_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<StoredObject>, ApiError> {
    let stored_object = state.service.complete_upload(identity, file_id).await?;

    Ok(Response::OK(stored_object))
}
