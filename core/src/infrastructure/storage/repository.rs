use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
};
use tracing::error;
use uuid::Uuid;

use crate::domain::{
    common::entities::app_errors::CoreError,
    storage::{
        entities::StoredObject,
        ports::StoredObjectRepository,
        value_objects::{CreateStoredObject, OffsetLimit, Paginated, StoredObjectFilter},
    },
};
use crate::entity::stored_objects::{
    ActiveModel as StoredObjectActiveModel, Column as StoredObjectColumn,
    Entity as StoredObjectEntity,
};

#[derive(Debug, Clone)]
pub struct PostgresStoredObjectRepository {
    pub db: DatabaseConnection,
}

impl PostgresStoredObjectRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl StoredObjectRepository for PostgresStoredObjectRepository {
    async fn create(&self, input: CreateStoredObject) -> Result<StoredObject, CoreError> {
        let stored_object = StoredObject::new(
            input.realm_id,
            input.bucket,
            input.object_key,
            input.original_name,
            input.mime_type,
            input.size_bytes,
            input.checksum_sha256,
            input.metadata,
            input.uploaded_by,
        );

        let active_model = StoredObjectActiveModel {
            id: Set(stored_object.id),
            realm_id: Set(stored_object.realm_id),
            bucket: Set(stored_object.bucket.clone()),
            object_key: Set(stored_object.object_key.clone()),
            original_name: Set(stored_object.original_name.clone()),
            mime_type: Set(stored_object.mime_type.clone()),
            size_bytes: Set(stored_object.size_bytes),
            checksum_sha256: Set(stored_object.checksum_sha256.clone()),
            metadata: Set(Some(stored_object.metadata.clone())),
            uploaded_by: Set(stored_object.uploaded_by),
            created_at: Set(stored_object.created_at.fixed_offset()),
            updated_at: Set(stored_object.updated_at.fixed_offset()),
            created_by: Set(stored_object.created_by),
            updated_by: Set(stored_object.updated_by),
        };

        active_model.insert(&self.db).await.map_err(|e| {
            error!("Failed to create stored object: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(stored_object)
    }

    async fn list(
        &self,
        filter: StoredObjectFilter,
        pagination: OffsetLimit,
    ) -> Result<Paginated<StoredObject>, CoreError> {
        let mut query = StoredObjectEntity::find();

        // Apply filters
        if let Some(realm_id) = filter.realm_id {
            query = query.filter(StoredObjectColumn::RealmId.eq(realm_id));
        }

        if let Some(mime_type) = filter.mime_type {
            query = query.filter(StoredObjectColumn::MimeType.eq(mime_type));
        }

        if let Some(uploaded_by) = filter.uploaded_by {
            query = query.filter(StoredObjectColumn::UploadedBy.eq(uploaded_by));
        }

        if let Some(created_before) = filter.created_before {
            query = query.filter(StoredObjectColumn::CreatedAt.lte(created_before));
        }

        if let Some(created_after) = filter.created_after {
            query = query.filter(StoredObjectColumn::CreatedAt.gte(created_after));
        }

        // Get total count
        let count = query.clone().count(&self.db).await.map_err(|e| {
            error!("Failed to count stored objects: {}", e);
            CoreError::InternalServerError
        })?;

        // Apply pagination and ordering
        query = query
            .order_by_desc(StoredObjectColumn::CreatedAt)
            .order_by_desc(StoredObjectColumn::Id)
            .limit(pagination.limit as u64)
            .offset(pagination.offset as u64);

        let models = query.all(&self.db).await.map_err(|e| {
            error!("Failed to list stored objects: {}", e);
            CoreError::InternalServerError
        })?;

        let items = models
            .iter()
            .map(StoredObject::from)
            .collect::<Vec<StoredObject>>();

        Ok(Paginated {
            items,
            offset: pagination.offset,
            limit: pagination.limit,
            count: count as i64,
        })
    }

    async fn get_by_id(&self, id: Uuid) -> Result<StoredObject, CoreError> {
        let model = StoredObjectEntity::find()
            .filter(StoredObjectColumn::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get stored object by id: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        Ok(StoredObject::from(&model))
    }

    async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        StoredObjectEntity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete stored object: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }
}
