use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DeviceProfile {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub device_id: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

impl DeviceProfile {
    pub fn new(realm_id: Uuid, device_id: String, user_id: Uuid, created_by: Option<Uuid>) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            device_id,
            user_id,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }
}
