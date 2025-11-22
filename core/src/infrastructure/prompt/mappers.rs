use chrono::{TimeZone, Utc};

use crate::domain::prompt::entities::prompt::Prompt;
use crate::entity::prompts::Model as PromptModel;

impl From<PromptModel> for Prompt {
    fn from(model: PromptModel) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);
        let deleted_at = model.deleted_at.map(|dt| dt.and_utc());
        Prompt {
            id: model.id,
            realm_id: model.realm_id,
            name: model.name,
            description: model.description,
            template: model.template,
            version: model.version,
            is_active: model.is_active,
            is_deleted: model.is_deleted,
            created_at,
            updated_at,
            deleted_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
            deleted_by: model.deleted_by,
        }
    }
}

impl From<&PromptModel> for Prompt {
    fn from(model: &PromptModel) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);
        let deleted_at = model.deleted_at.map(|dt| dt.and_utc());
        Prompt {
            id: model.id,
            realm_id: model.realm_id,
            name: model.name.clone(),
            description: model.description.clone(),
            template: model.template.clone(),
            version: model.version.clone(),
            is_active: model.is_active,
            is_deleted: model.is_deleted,
            created_at,
            updated_at,
            deleted_at,
            created_by: model.created_by,
            updated_by: model.updated_by,
            deleted_by: model.deleted_by,
        }
    }
}
