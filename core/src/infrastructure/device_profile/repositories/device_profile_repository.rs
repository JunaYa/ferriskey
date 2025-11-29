use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        device_profile::{entities::DeviceProfile, ports::DeviceProfileRepository},
    },
    entity::device_profiles::{ActiveModel, Column, Entity},
};

#[derive(Debug, Clone)]
pub struct PostgresDeviceProfileRepository {
    pub db: DatabaseConnection,
}

impl PostgresDeviceProfileRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl DeviceProfileRepository for PostgresDeviceProfileRepository {
    async fn get_by_realm_and_device(
        &self,
        realm_id: Uuid,
        device_id: &str,
    ) -> Result<Option<DeviceProfile>, CoreError> {
        let profile = Entity::find()
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::DeviceId.eq(device_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get device profile: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(profile.map(DeviceProfile::from))
    }

    async fn create(&self, profile: DeviceProfile) -> Result<DeviceProfile, CoreError> {
        let active_model = ActiveModel {
            id: Set(profile.id),
            realm_id: Set(profile.realm_id),
            device_id: Set(profile.device_id.clone()),
            user_id: Set(profile.user_id),
            created_at: Set(profile.created_at.fixed_offset()),
            updated_at: Set(profile.updated_at.fixed_offset()),
            created_by: Set(profile.created_by),
            updated_by: Set(profile.updated_by),
        };

        let created = Entity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to create device profile: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(DeviceProfile::from(created))
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<DeviceProfile>, CoreError> {
        let profile = Entity::find_by_id(id).one(&self.db).await.map_err(|e| {
            error!("Failed to get device profile by id: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(profile.map(DeviceProfile::from))
    }
}
