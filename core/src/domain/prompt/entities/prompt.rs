use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd, ToSchema)]
pub struct Prompt {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub description: String,
    pub template: String,
    pub version: String,
    pub is_active: bool,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub updated_by: Uuid,
    pub deleted_by: Option<Uuid>,
}

impl Prompt {
    pub fn new(
        realm_id: Uuid,
        name: String,
        description: String,
        template: String,
        version: String,
        created_by: Uuid,
    ) -> Self {
        let (_, timestamp) = generate_timestamp();
        let now = Utc::now();

        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            name,
            description,
            template,
            version,
            is_active: true,
            is_deleted: false,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            created_by,
            updated_by: created_by,
            deleted_by: None,
        }
    }

    pub fn update(
        &mut self,
        name: Option<String>,
        description: Option<String>,
        template: Option<String>,
        version: Option<String>,
        is_active: Option<bool>,
        updated_by: Uuid,
    ) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(description) = description {
            self.description = description;
        }
        if let Some(template) = template {
            self.template = template;
        }
        if let Some(version) = version {
            self.version = version;
        }
        if let Some(is_active) = is_active {
            self.is_active = is_active;
        }
        self.updated_by = updated_by;
        self.updated_at = Utc::now();
    }

    pub fn soft_delete(&mut self, deleted_by: Uuid) {
        self.is_deleted = true;
        self.deleted_at = Some(Utc::now());
        self.deleted_by = Some(deleted_by);
        self.updated_at = Utc::now();
        self.updated_by = deleted_by;
    }
}
