use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use tracing::error;
use uuid::Uuid;

use crate::domain::prompt::value_objects::GetPromptsFilter;
use crate::domain::{
    common::entities::app_errors::CoreError,
    prompt::{entities::prompt::Prompt, ports::PromptRepository},
};
use crate::entity::prompts::{
    self, ActiveModel as PromptActiveModel, Column as PromptColumn, Entity as PromptEntity,
};

#[derive(Debug, Clone)]
pub struct PostgresPromptRepository {
    pub db: DatabaseConnection,
}

impl PostgresPromptRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PromptRepository for PostgresPromptRepository {
    async fn fetch_prompts_by_realm(
        &self,
        realm_id: Uuid,
        filter: GetPromptsFilter,
    ) -> Result<Vec<Prompt>, CoreError> {
        let mut query = prompts::Entity::find().filter(prompts::Column::RealmId.eq(realm_id));

        if !filter.include_deleted {
            query = query.filter(PromptColumn::IsDeleted.eq(false));
        }

        if let Some(name) = filter.name {
            query = query.filter(PromptColumn::Name.eq(name));
        }

        if let Some(description) = filter.description {
            query = query.filter(PromptColumn::Description.eq(description));
        }

        query = query.order_by_desc(prompts::Column::CreatedAt);

        if let Some(limit) = filter.limit {
            query = query.limit(limit as u64);
        }

        if let Some(offset) = filter.offset {
            query = query.offset(offset as u64);
        }

        let prompts = query
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to fetch prompts by realm: {}", e);
                CoreError::InternalServerError
            })?
            .iter()
            .map(Prompt::from)
            .collect::<Vec<Prompt>>();

        Ok(prompts)
    }

    async fn get_prompt_by_id(
        &self,
        prompt_id: Uuid,
        realm_id: Uuid,
    ) -> Result<Option<Prompt>, CoreError> {
        let prompt = PromptEntity::find()
            .filter(PromptColumn::Id.eq(prompt_id))
            .filter(PromptColumn::RealmId.eq(realm_id))
            .filter(PromptColumn::IsDeleted.eq(false))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get prompt by id: {}", e);
                CoreError::InternalServerError
            })?
            .map(Prompt::from);

        Ok(prompt)
    }

    async fn create_prompt(&self, prompt: Prompt) -> Result<Prompt, CoreError> {
        let created_prompt = PromptEntity::insert(PromptActiveModel {
            id: Set(prompt.id),
            realm_id: Set(prompt.realm_id),
            name: Set(prompt.name),
            description: Set(prompt.description),
            template: Set(prompt.template),
            version: Set(prompt.version),
            is_active: Set(prompt.is_active),
            is_deleted: Set(prompt.is_deleted),
            created_at: Set(prompt.created_at.naive_utc()),
            updated_at: Set(prompt.updated_at.naive_utc()),
            deleted_at: Set(prompt.deleted_at.map(|dt| dt.naive_utc())),
            created_by: Set(prompt.created_by),
            updated_by: Set(prompt.updated_by),
            deleted_by: Set(prompt.deleted_by),
        })
        .exec_with_returning(&self.db)
        .await
        .map(Prompt::from)
        .map_err(|e| {
            error!("Failed to create prompt: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(created_prompt)
    }

    async fn update_prompt(&self, prompt: Prompt) -> Result<Prompt, CoreError> {
        let updated_prompt = PromptEntity::update(PromptActiveModel {
            id: Set(prompt.id),
            realm_id: Set(prompt.realm_id),
            name: Set(prompt.name),
            description: Set(prompt.description),
            template: Set(prompt.template),
            version: Set(prompt.version),
            is_active: Set(prompt.is_active),
            is_deleted: Set(prompt.is_deleted),
            created_at: Set(prompt.created_at.naive_utc()),
            updated_at: Set(prompt.updated_at.naive_utc()),
            deleted_at: Set(prompt.deleted_at.map(|dt| dt.naive_utc())),
            created_by: Set(prompt.created_by),
            updated_by: Set(prompt.updated_by),
            deleted_by: Set(prompt.deleted_by),
        })
        .filter(PromptColumn::Id.eq(prompt.id))
        .filter(PromptColumn::RealmId.eq(prompt.realm_id))
        .exec(&self.db)
        .await
        .map(Prompt::from)
        .map_err(|e| {
            error!("Failed to update prompt: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(updated_prompt)
    }

    async fn delete_prompt(
        &self,
        prompt_id: Uuid,
        realm_id: Uuid,
        deleted_by: Uuid,
    ) -> Result<(), CoreError> {
        let now = Utc::now().naive_utc();

        PromptEntity::update_many()
            .col_expr(
                PromptColumn::IsDeleted,
                sea_orm::sea_query::Expr::value(true),
            )
            .col_expr(
                PromptColumn::DeletedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .col_expr(
                PromptColumn::DeletedBy,
                sea_orm::sea_query::Expr::value(deleted_by),
            )
            .col_expr(
                PromptColumn::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(PromptColumn::Id.eq(prompt_id))
            .filter(PromptColumn::RealmId.eq(realm_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete prompt: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }
}
