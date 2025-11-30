use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order, QueryFilter,
    QueryOrder, QuerySelect,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        food_reaction::{
            entities::FoodReaction, ports::FoodReactionRepository,
            value_objects::GetFoodReactionFilter,
        },
    },
    entity::{
        food_reaction_symptoms::{
            ActiveModel as SymptomActiveModel, Column as SymptomColumn, Entity as SymptomEntity,
        },
        food_reactions::{ActiveModel, Column, Entity},
    },
    infrastructure::food_reaction::mappers::map_symptoms,
};

#[derive(Debug, Clone)]
pub struct PostgresFoodReactionRepository {
    pub db: DatabaseConnection,
}

impl PostgresFoodReactionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl FoodReactionRepository for PostgresFoodReactionRepository {
    async fn create_reaction(
        &self,
        reaction: FoodReaction,
        symptoms: Vec<String>,
    ) -> Result<FoodReaction, CoreError> {
        // Create reaction
        let active_model = ActiveModel {
            id: Set(reaction.id),
            realm_id: Set(reaction.realm_id),
            device_id: Set(reaction.device_id.clone()),
            user_id: Set(reaction.user_id),
            analysis_item_id: Set(reaction.analysis_item_id),
            eaten_at: Set(reaction.eaten_at.fixed_offset()),
            feeling: Set(reaction.feeling.clone()),
            symptom_onset: Set(reaction.symptom_onset.clone()),
            notes: Set(reaction.notes.clone()),
            created_at: Set(reaction.created_at.fixed_offset()),
            updated_at: Set(reaction.updated_at.fixed_offset()),
            created_by: Set(reaction.created_by),
            updated_by: Set(reaction.updated_by),
        };

        let created = Entity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to create food reaction: {}", e);
                CoreError::InternalServerError
            })?;

        // Create symptoms
        if !symptoms.is_empty() {
            let symptom_models: Vec<SymptomActiveModel> = symptoms
                .into_iter()
                .map(|symptom_code| {
                    let (now, timestamp) = crate::domain::common::generate_timestamp();
                    SymptomActiveModel {
                        id: Set(Uuid::new_v7(timestamp)),
                        reaction_id: Set(created.id),
                        symptom_code: Set(symptom_code),
                        created_at: Set(now.fixed_offset()),
                        updated_at: Set(now.fixed_offset()),
                        created_by: Set(reaction.created_by),
                        updated_by: Set(reaction.created_by),
                    }
                })
                .collect();

            SymptomEntity::insert_many(symptom_models)
                .exec(&self.db)
                .await
                .map_err(|e| {
                    error!("Failed to create food reaction symptoms: {}", e);
                    CoreError::InternalServerError
                })?;
        }

        let mut result = FoodReaction::from(created);
        // Load symptoms
        let symptom_models = SymptomEntity::find()
            .filter(SymptomColumn::ReactionId.eq(result.id))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to load food reaction symptoms: {}", e);
                CoreError::InternalServerError
            })?;
        result.symptoms = map_symptoms(symptom_models);

        Ok(result)
    }

    async fn get_by_id(
        &self,
        reaction_id: Uuid,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<FoodReaction>, CoreError> {
        let reaction = Entity::find()
            .filter(Column::Id.eq(reaction_id))
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get food reaction: {}", e);
                CoreError::InternalServerError
            })?;

        if let Some(reaction_model) = reaction {
            // Load symptoms
            let symptom_models = SymptomEntity::find()
                .filter(SymptomColumn::ReactionId.eq(reaction_model.id))
                .all(&self.db)
                .await
                .map_err(|e| {
                    error!("Failed to load food reaction symptoms: {}", e);
                    CoreError::InternalServerError
                })?;

            let mut result = FoodReaction::from(reaction_model);
            result.symptoms = map_symptoms(symptom_models);
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    async fn get_by_realm(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetFoodReactionFilter,
    ) -> Result<Vec<FoodReaction>, CoreError> {
        let mut query = Entity::find()
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::UserId.eq(user_id));

        // Apply filters
        let mut condition = Condition::all();

        if let Some(ref feeling) = filter.feeling {
            condition = condition.add(Column::Feeling.eq(feeling.clone()));
        }

        if let Some(ref feelings) = filter.feeling_in
            && !feelings.is_empty()
        {
            condition = condition.add(Column::Feeling.is_in(feelings.clone()));
        }

        if let Some(analysis_item_id) = filter.analysis_item_id {
            condition = condition.add(Column::AnalysisItemId.eq(analysis_item_id));
        }

        if let Some(ref symptom_onset) = filter.symptom_onset {
            condition = condition.add(Column::SymptomOnset.eq(symptom_onset.clone()));
        }

        if let Some(eaten_at_gte) = filter.eaten_at_gte {
            condition = condition.add(Column::EatenAt.gte(eaten_at_gte.fixed_offset()));
        }

        if let Some(eaten_at_lte) = filter.eaten_at_lte {
            condition = condition.add(Column::EatenAt.lte(eaten_at_lte.fixed_offset()));
        }

        if let Some(created_at_gte) = filter.created_at_gte {
            condition = condition.add(Column::CreatedAt.gte(created_at_gte.fixed_offset()));
        }

        if let Some(created_at_lte) = filter.created_at_lte {
            condition = condition.add(Column::CreatedAt.lte(created_at_lte.fixed_offset()));
        }

        // has_symptoms filter: check if reaction has any symptoms
        if let Some(has_symptoms) = filter.has_symptoms {
            if has_symptoms {
                // Reaction must have at least one symptom
                let reactions_with_symptoms = SymptomEntity::find()
                    .select_only()
                    .column(SymptomColumn::ReactionId)
                    .all(&self.db)
                    .await
                    .map_err(|e| {
                        error!("Failed to get reactions with symptoms: {}", e);
                        CoreError::InternalServerError
                    })?;
                let reaction_ids: Vec<Uuid> = reactions_with_symptoms
                    .iter()
                    .map(|s| s.reaction_id)
                    .collect();
                if !reaction_ids.is_empty() {
                    condition = condition.add(Column::Id.is_in(reaction_ids));
                } else {
                    // No reactions have symptoms, return empty
                    return Ok(Vec::new());
                }
            } else {
                // Reaction must have no symptoms
                let reactions_with_symptoms = SymptomEntity::find()
                    .select_only()
                    .column(SymptomColumn::ReactionId)
                    .all(&self.db)
                    .await
                    .map_err(|e| {
                        error!("Failed to get reactions with symptoms: {}", e);
                        CoreError::InternalServerError
                    })?;
                let reaction_ids: Vec<Uuid> = reactions_with_symptoms
                    .iter()
                    .map(|s| s.reaction_id)
                    .collect();
                if !reaction_ids.is_empty() {
                    condition = condition.add(Column::Id.is_not_in(reaction_ids));
                }
            }
        }

        query = query.filter(condition);

        // Apply sorting
        if let Some(ref sort_str) = filter.sort {
            for sort_part in sort_str.split(',') {
                let sort_part = sort_part.trim();
                if let Some(field) = sort_part.strip_prefix('-') {
                    match field {
                        "eaten_at" => {
                            query = query.order_by(Column::EatenAt, Order::Desc);
                        }
                        "created_at" => {
                            query = query.order_by(Column::CreatedAt, Order::Desc);
                        }
                        "feeling" => {
                            query = query.order_by(Column::Feeling, Order::Desc);
                        }
                        _ => {}
                    }
                } else {
                    match sort_part {
                        "eaten_at" => {
                            query = query.order_by(Column::EatenAt, Order::Asc);
                        }
                        "created_at" => {
                            query = query.order_by(Column::CreatedAt, Order::Asc);
                        }
                        "feeling" => {
                            query = query.order_by(Column::Feeling, Order::Asc);
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // Default sort: -eaten_at
            query = query.order_by_desc(Column::EatenAt);
        }

        // Apply pagination
        if let Some(limit) = filter.limit {
            query = query.limit(limit as u64);
        }

        if let Some(offset) = filter.offset {
            query = query.offset(offset as u64);
        }

        let reactions = query.all(&self.db).await.map_err(|e| {
            error!("Failed to get food reactions: {}", e);
            CoreError::InternalServerError
        })?;

        // Load symptoms for all reactions
        let reaction_ids: Vec<Uuid> = reactions.iter().map(|r| r.id).collect();
        let all_symptoms = if !reaction_ids.is_empty() {
            SymptomEntity::find()
                .filter(SymptomColumn::ReactionId.is_in(reaction_ids))
                .all(&self.db)
                .await
                .map_err(|e| {
                    error!("Failed to load food reaction symptoms: {}", e);
                    CoreError::InternalServerError
                })?
        } else {
            Vec::new()
        };

        // Group symptoms by reaction_id
        use std::collections::HashMap;
        let mut symptoms_map: HashMap<Uuid, Vec<String>> = HashMap::new();
        for symptom in all_symptoms {
            symptoms_map
                .entry(symptom.reaction_id)
                .or_default()
                .push(symptom.symptom_code);
        }

        // Build result
        let result: Vec<FoodReaction> = reactions
            .iter()
            .map(|r| {
                let mut reaction = FoodReaction::from(r);
                reaction.symptoms = symptoms_map.remove(&reaction.id).unwrap_or_default();
                reaction
            })
            .collect();

        Ok(result)
    }

    async fn update_reaction(
        &self,
        reaction: FoodReaction,
        symptoms: Vec<String>,
    ) -> Result<FoodReaction, CoreError> {
        // Update reaction
        let active_model = ActiveModel {
            id: Set(reaction.id),
            realm_id: Set(reaction.realm_id),
            device_id: Set(reaction.device_id.clone()),
            user_id: Set(reaction.user_id),
            analysis_item_id: Set(reaction.analysis_item_id),
            eaten_at: Set(reaction.eaten_at.fixed_offset()),
            feeling: Set(reaction.feeling.clone()),
            symptom_onset: Set(reaction.symptom_onset.clone()),
            notes: Set(reaction.notes.clone()),
            created_at: Set(reaction.created_at.fixed_offset()),
            updated_at: Set(reaction.updated_at.fixed_offset()),
            created_by: Set(reaction.created_by),
            updated_by: Set(reaction.updated_by),
        };

        Entity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to update food reaction: {}", e);
                CoreError::InternalServerError
            })?;

        // Delete existing symptoms
        SymptomEntity::delete_many()
            .filter(SymptomColumn::ReactionId.eq(reaction.id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete food reaction symptoms: {}", e);
                CoreError::InternalServerError
            })?;

        // Create new symptoms
        if !symptoms.is_empty() {
            let symptom_models: Vec<SymptomActiveModel> = symptoms
                .into_iter()
                .map(|symptom_code| {
                    let (now, timestamp) = crate::domain::common::generate_timestamp();
                    SymptomActiveModel {
                        id: Set(Uuid::new_v7(timestamp)),
                        reaction_id: Set(reaction.id),
                        symptom_code: Set(symptom_code),
                        created_at: Set(now.fixed_offset()),
                        updated_at: Set(now.fixed_offset()),
                        created_by: Set(reaction.updated_by),
                        updated_by: Set(reaction.updated_by),
                    }
                })
                .collect();

            SymptomEntity::insert_many(symptom_models)
                .exec(&self.db)
                .await
                .map_err(|e| {
                    error!("Failed to create food reaction symptoms: {}", e);
                    CoreError::InternalServerError
                })?;
        }

        // Reload reaction with symptoms
        self.get_by_id(reaction.id, reaction.realm_id, reaction.user_id)
            .await
            .map(|opt| opt.expect("Reaction should exist after update"))
    }

    async fn delete_reaction(
        &self,
        reaction_id: Uuid,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), CoreError> {
        // Delete symptoms first
        SymptomEntity::delete_many()
            .filter(SymptomColumn::ReactionId.eq(reaction_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete food reaction symptoms: {}", e);
                CoreError::InternalServerError
            })?;

        // Delete reaction
        Entity::delete_many()
            .filter(Column::Id.eq(reaction_id))
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::UserId.eq(user_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete food reaction: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }
}
