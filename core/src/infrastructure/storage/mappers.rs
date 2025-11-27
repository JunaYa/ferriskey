use chrono::Utc;

use crate::domain::storage::entities::StoredObject;
use crate::entity::stored_objects::Model as StoredObjectModel;

impl From<&StoredObjectModel> for StoredObject {
    fn from(model: &StoredObjectModel) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id,
            bucket: model.bucket.clone(),
            object_key: model.object_key.clone(),
            original_name: model.original_name.clone(),
            mime_type: model.mime_type.clone(),
            size_bytes: model.size_bytes,
            checksum_sha256: model.checksum_sha256.clone(),
            metadata: model.metadata.clone().unwrap_or_default(),
            uploaded_by: model.uploaded_by,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}
