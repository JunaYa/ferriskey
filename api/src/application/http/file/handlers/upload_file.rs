use axum::{
    Extension,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response as AxumResponse},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    storage::{entities::StoredObject, services::FileService},
};
use tracing::{error, warn};

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50 MB

#[utoipa::path(
    post,
    path = "/upload",
    tag = "file",
    summary = "Upload a file directly",
    description = "Upload a file directly via multipart form data. The file will be uploaded to MinIO and metadata stored in the database.",
    responses(
        (status = 200, description = "File uploaded successfully", body = StoredObject),
        (status = 400, description = "Bad request"),
        (status = 403, description = "Forbidden"),
        (status = 413, description = "File too large")
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
)]
pub async fn upload_file(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    mut multipart: Multipart,
) -> Result<AxumResponse, ApiError> {
    let mut filename: Option<String> = None;
    let mut mime_type: Option<String> = None;
    let mut file_data: Option<bytes::Bytes> = None;
    let mut metadata: Option<serde_json::Value> = None;

    // Parse multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        ApiError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                // Get filename
                if let Some(file_name) = field.file_name() {
                    filename = Some(file_name.to_string());
                } else {
                    return Err(ApiError::BadRequest(
                        "Missing filename in file field".to_string(),
                    ));
                }

                // Get content type
                if let Some(content_type) = field.content_type() {
                    mime_type = Some(content_type.to_string());
                }

                // Read file data
                let data = field.bytes().await.map_err(|e| {
                    error!("Failed to read file bytes: {}", e);
                    ApiError::BadRequest(format!("Failed to read file: {}", e))
                })?;

                // Validate file is not empty
                if data.is_empty() {
                    warn!(
                        filename = %filename.as_ref().unwrap_or(&"unknown".to_string()),
                        "Empty file upload attempted"
                    );
                    return Err(ApiError::BadRequest("File cannot be empty".to_string()));
                }

                // Validate file size
                if data.len() > MAX_FILE_SIZE {
                    return Ok((
                        StatusCode::PAYLOAD_TOO_LARGE,
                        format!("File too large. Max size is {} bytes", MAX_FILE_SIZE),
                    )
                        .into_response());
                }

                file_data = Some(data);
            }
            "metadata" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiError::BadRequest(format!("Failed to read metadata: {}", e)))?;
                metadata =
                    Some(serde_json::from_str(&text).map_err(|e| {
                        ApiError::BadRequest(format!("Invalid metadata JSON: {}", e))
                    })?);
            }
            _ => {
                // Ignore unknown fields
            }
        }
    }

    // Validate required fields
    let filename = filename.ok_or_else(|| {
        ApiError::BadRequest("Missing 'file' field in multipart form".to_string())
    })?;

    let file_data = file_data
        .ok_or_else(|| ApiError::BadRequest("Missing file data in 'file' field".to_string()))?;

    let mime_type = mime_type.unwrap_or_else(|| "application/octet-stream".to_string());
    let metadata = metadata.unwrap_or_default();

    // Upload file
    let stored_object = state
        .service
        .upload_file_direct(
            identity,
            realm_name.clone(),
            filename.clone(),
            mime_type.clone(),
            file_data,
            metadata,
        )
        .await
        .map_err(|e| {
            error!(
                error = %e,
                realm_name = %realm_name,
                filename = %filename,
                mime_type = %mime_type,
                "Failed to upload file =>>>>"
            );
            ApiError::from(e)
        })?;

    Ok(Response::OK(stored_object).into_response())
}
