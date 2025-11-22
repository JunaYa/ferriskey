use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    prompt::{
        entities::prompt::Prompt,
        value_objects::{
            CreatePromptInput, DeletePromptInput, GetPromptInput, GetPromptsFilter,
            UpdatePromptInput,
        },
    },
    realm::entities::Realm,
};

#[cfg_attr(test, mockall::automock)]
pub trait PromptService: Send + Sync {
    fn get_prompts(
        &self,
        identity: Identity,
        input: GetPromptsFilter,
    ) -> impl Future<Output = Result<Vec<Prompt>, CoreError>> + Send;

    fn get_prompt(
        &self,
        identity: Identity,
        input: GetPromptInput,
    ) -> impl Future<Output = Result<Option<Prompt>, CoreError>> + Send;

    fn create_prompt(
        &self,
        identity: Identity,
        input: CreatePromptInput,
    ) -> impl Future<Output = Result<Prompt, CoreError>> + Send;

    fn update_prompt(
        &self,
        identity: Identity,
        input: UpdatePromptInput,
    ) -> impl Future<Output = Result<Prompt, CoreError>> + Send;

    fn delete_prompt(
        &self,
        identity: Identity,
        input: DeletePromptInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait PromptRepository: Send + Sync {
    fn fetch_prompts_by_realm(
        &self,
        realm_id: Uuid,
        filter: GetPromptsFilter,
    ) -> impl Future<Output = Result<Vec<Prompt>, CoreError>> + Send;

    fn get_prompt_by_id(
        &self,
        prompt_id: Uuid,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<Prompt>, CoreError>> + Send;

    fn create_prompt(
        &self,
        prompt: Prompt,
    ) -> impl Future<Output = Result<Prompt, CoreError>> + Send;

    fn update_prompt(
        &self,
        prompt: Prompt,
    ) -> impl Future<Output = Result<Prompt, CoreError>> + Send;

    fn delete_prompt(
        &self,
        prompt_id: Uuid,
        realm_id: Uuid,
        deleted_by: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait PromptPolicy: Send + Sync {
    fn can_create_prompt(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_prompt(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_delete_prompt(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_prompt(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
