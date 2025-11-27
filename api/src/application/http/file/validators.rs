use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct InitiateUploadRequest {
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    pub use_presigned: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ListFilesQuery {
    pub realm_id: Option<Uuid>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub mime_type: Option<String>,
    pub uploaded_by: Option<Uuid>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}
