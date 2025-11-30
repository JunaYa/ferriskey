use sea_orm::{
    ActiveValue::Set,
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order, QueryFilter, QueryOrder,
    QuerySelect,
    prelude::Expr,
    sea_query::{IntoCondition, extension::postgres::PgExpr},
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::entities::app_errors::CoreError,
        food_analysis::{
            entities::FoodAnalysisTrigger,
            ports::FoodAnalysisTriggerRepository,
            value_objects::{
                GetFoodAnalysisTriggerFilter, GetTriggerCategoryFilter, TriggerCategoryStats,
            },
        },
    },
    entity::food_analysis_triggers::{ActiveModel, Column, Entity},
};

#[derive(Debug, Clone)]
pub struct PostgresFoodAnalysisTriggerRepository {
    pub db: DatabaseConnection,
}

impl PostgresFoodAnalysisTriggerRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl FoodAnalysisTriggerRepository for PostgresFoodAnalysisTriggerRepository {
    async fn create_trigger(
        &self,
        trigger: FoodAnalysisTrigger,
    ) -> Result<FoodAnalysisTrigger, CoreError> {
        let active_model = ActiveModel {
            id: Set(trigger.id),
            realm_id: Set(trigger.realm_id),
            item_id: Set(trigger.item_id),
            ingredient_name: Set(trigger.ingredient_name.clone()),
            trigger_category: Set(trigger.trigger_category.clone()),
            risk_level: Set(trigger.risk_level.clone()),
            risk_reason: Set(trigger.risk_reason.clone()),
            created_at: Set(trigger.created_at.fixed_offset()),
            updated_at: Set(trigger.updated_at.fixed_offset()),
            created_by: Set(trigger.created_by),
            updated_by: Set(trigger.updated_by),
        };

        let created = Entity::insert(active_model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to create food analysis trigger: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(FoodAnalysisTrigger::from(created))
    }

    async fn create_triggers_batch(
        &self,
        triggers: Vec<FoodAnalysisTrigger>,
    ) -> Result<Vec<FoodAnalysisTrigger>, CoreError> {
        let mut created_triggers = Vec::new();

        for trigger in triggers {
            let created = self.create_trigger(trigger).await?;
            created_triggers.push(created);
        }

        Ok(created_triggers)
    }

    async fn get_by_item_id(
        &self,
        item_id: Uuid,
        realm_id: Uuid,
        filter: GetFoodAnalysisTriggerFilter,
    ) -> Result<Vec<FoodAnalysisTrigger>, CoreError> {
        let mut query = Entity::find()
            .filter(Column::ItemId.eq(item_id))
            .filter(Column::RealmId.eq(realm_id));

        // Apply filters
        let mut condition = Condition::all();

        if let Some(ref trigger_category) = filter.trigger_category {
            condition = condition.add(Column::TriggerCategory.eq(trigger_category.clone()));
        }

        if let Some(ref risk_level) = filter.risk_level {
            condition = condition.add(Column::RiskLevel.eq(risk_level.clone()));
        }

        if let Some(ref risk_levels) = filter.risk_level_in
            && !risk_levels.is_empty()
        {
            condition = condition.add(Column::RiskLevel.is_in(risk_levels.clone()));
        }

        if let Some(ref ingredient_name) = filter.ingredient_name_ilike {
            condition = condition.add(
                Expr::col(Column::IngredientName)
                    .ilike(format!("%{}%", ingredient_name))
                    .into_condition(),
            );
        }

        query = query.filter(condition);

        // Apply sorting
        if let Some(ref sort_str) = filter.sort {
            // Parse sort string like "risk_level,-created_at"
            for sort_part in sort_str.split(',') {
                let sort_part = sort_part.trim();
                if let Some(field) = sort_part.strip_prefix('-') {
                    match field {
                        "risk_level" => {
                            query = query.order_by(Column::RiskLevel, Order::Desc);
                        }
                        "ingredient_name" => {
                            query = query.order_by(Column::IngredientName, Order::Desc);
                        }
                        "trigger_category" => {
                            query = query.order_by(Column::TriggerCategory, Order::Desc);
                        }
                        "created_at" => {
                            query = query.order_by(Column::CreatedAt, Order::Desc);
                        }
                        _ => {}
                    }
                } else {
                    match sort_part {
                        "risk_level" => {
                            query = query.order_by(Column::RiskLevel, Order::Asc);
                        }
                        "ingredient_name" => {
                            query = query.order_by(Column::IngredientName, Order::Asc);
                        }
                        "trigger_category" => {
                            query = query.order_by(Column::TriggerCategory, Order::Asc);
                        }
                        "created_at" => {
                            query = query.order_by(Column::CreatedAt, Order::Asc);
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // Default sort: risk_level,-created_at
            query = query
                .order_by(Column::RiskLevel, Order::Asc)
                .order_by(Column::CreatedAt, Order::Desc);
        }

        // Apply pagination
        if let Some(limit) = filter.limit {
            query = query.limit(limit as u64);
        }

        if let Some(offset) = filter.offset {
            query = query.offset(offset as u64);
        }

        let triggers = query
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get food analysis triggers: {}", e);
                CoreError::InternalServerError
            })?
            .iter()
            .map(FoodAnalysisTrigger::from)
            .collect();

        Ok(triggers)
    }

    async fn get_categories_stats(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetTriggerCategoryFilter,
    ) -> Result<Vec<TriggerCategoryStats>, CoreError> {
        use crate::entity::{food_analysis_items, food_analysis_requests};
        use std::collections::HashMap;

        // Query all triggers for the user
        // We need to join through items to get to requests, then filter by user_id
        // First, get all item_ids for this user
        let user_items = food_analysis_items::Entity::find()
            .inner_join(food_analysis_requests::Entity)
            .filter(food_analysis_items::Column::RealmId.eq(realm_id))
            .filter(food_analysis_requests::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get user items for trigger stats: {}", e);
                CoreError::InternalServerError
            })?;

        let item_ids: Vec<Uuid> = user_items.iter().map(|item| item.id).collect();

        if item_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Now get all triggers for these items
        let all_triggers = Entity::find()
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::ItemId.is_in(item_ids))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get trigger categories stats: {}", e);
                CoreError::InternalServerError
            })?;

        // Apply category filter
        let filtered_triggers: Vec<_> = all_triggers
            .iter()
            .filter(|t| {
                if let Some(ref category) = filter.trigger_category {
                    t.trigger_category == *category
                } else if let Some(ref categories) = filter.trigger_category_in {
                    categories.contains(&t.trigger_category)
                } else if let Some(ref category_pattern) = filter.trigger_category_ilike {
                    t.trigger_category
                        .to_lowercase()
                        .contains(&category_pattern.to_lowercase())
                } else {
                    true
                }
            })
            .collect();

        // Aggregate by category
        let mut stats_map: HashMap<String, (i64, i64, i64, i64)> = HashMap::new();

        for trigger in filtered_triggers {
            let entry = stats_map
                .entry(trigger.trigger_category.clone())
                .or_insert((0, 0, 0, 0));
            entry.0 += 1; // total count
            match trigger.risk_level.as_str() {
                "HIGH" => entry.1 += 1,
                "MEDIUM" => entry.2 += 1,
                "LOW" => entry.3 += 1,
                _ => {}
            }
        }

        // Convert to Vec<TriggerCategoryStats>
        let mut stats: Vec<TriggerCategoryStats> = stats_map
            .into_iter()
            .map(
                |(category, (count, high, medium, low))| TriggerCategoryStats {
                    trigger_category: category,
                    count,
                    high_risk_count: high,
                    medium_risk_count: medium,
                    low_risk_count: low,
                },
            )
            .collect();

        // Apply sorting
        if let Some(ref sort_str) = filter.sort {
            for sort_part in sort_str.split(',') {
                let sort_part = sort_part.trim();
                let descending = sort_part.starts_with('-');
                let field = if descending {
                    sort_part.strip_prefix('-').unwrap_or(sort_part)
                } else {
                    sort_part
                };

                match field {
                    "trigger_category" => {
                        stats.sort_by(|a, b| {
                            if descending {
                                b.trigger_category.cmp(&a.trigger_category)
                            } else {
                                a.trigger_category.cmp(&b.trigger_category)
                            }
                        });
                    }
                    "count" => {
                        stats.sort_by(|a, b| {
                            if descending {
                                b.count.cmp(&a.count)
                            } else {
                                a.count.cmp(&b.count)
                            }
                        });
                    }
                    "high_risk_count" => {
                        stats.sort_by(|a, b| {
                            if descending {
                                b.high_risk_count.cmp(&a.high_risk_count)
                            } else {
                                a.high_risk_count.cmp(&b.high_risk_count)
                            }
                        });
                    }
                    _ => {}
                }
            }
        } else {
            // Default sort: -count
            stats.sort_by(|a, b| b.count.cmp(&a.count));
        }

        // Apply pagination
        let start = filter.offset.unwrap_or(0) as usize;
        let end = start + filter.limit.unwrap_or(20) as usize;
        let stats = stats.into_iter().skip(start).take(end - start).collect();

        Ok(stats)
    }
}
