use std::future::Future;
use uuid::Uuid;

use crate::domain::{
    common::entities::app_errors::CoreError,
    food_reaction::{entities::FoodReaction, value_objects::GetFoodReactionFilter},
};

/// Repository trait for food reactions
#[cfg_attr(test, mockall::automock)]
pub trait FoodReactionRepository: Send + Sync {
    fn create_reaction(
        &self,
        reaction: FoodReaction,
        symptoms: Vec<String>,
    ) -> impl Future<Output = Result<FoodReaction, CoreError>> + Send;

    fn get_by_id(
        &self,
        reaction_id: Uuid,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Option<FoodReaction>, CoreError>> + Send;

    fn get_by_realm(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetFoodReactionFilter,
    ) -> impl Future<Output = Result<Vec<FoodReaction>, CoreError>> + Send;

    fn update_reaction(
        &self,
        reaction: FoodReaction,
        symptoms: Vec<String>,
    ) -> impl Future<Output = Result<FoodReaction, CoreError>> + Send;

    fn delete_reaction(
        &self,
        reaction_id: Uuid,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
