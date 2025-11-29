use crate::{domain::device_profile::entities::DeviceProfile, entity::device_profiles};

impl From<&device_profiles::Model> for DeviceProfile {
    fn from(model: &device_profiles::Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id,
            device_id: model.device_id.clone(),
            user_id: model.user_id,
            created_at: model.created_at.to_utc(),
            updated_at: model.updated_at.to_utc(),
            created_by: model.created_by,
            updated_by: model.updated_by,
        }
    }
}

impl From<device_profiles::Model> for DeviceProfile {
    fn from(model: device_profiles::Model) -> Self {
        Self::from(&model)
    }
}
