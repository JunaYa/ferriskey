use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use tracing::error;
use uuid::Uuid;

use crate::domain::{
    common::entities::app_errors::CoreError,
    food_stats::{
        ports::FoodStatsRepository,
        value_objects::{
            GetSymptomStatsFilter, GetTimelineStatsFilter, OverviewStats, SafeFoodStats,
            SymptomStats, SymptomStatsResponse, TimelineStats, TimelineStatsResponse, TriggerStats,
        },
    },
};

#[derive(Debug, Clone)]
pub struct PostgresFoodStatsRepository {
    pub db: DatabaseConnection,
}

impl PostgresFoodStatsRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Map trigger category to emoji
    fn get_emoji_for_category(category: &str) -> Option<String> {
        // Simple mapping - can be extended
        match category.to_lowercase().as_str() {
            s if s.contains("dairy") => Some("🥛".to_string()),
            s if s.contains("garlic") => Some("🧄".to_string()),
            s if s.contains("coffee") => Some("☕️".to_string()),
            s if s.contains("vegetable") || s.contains("veggie") => Some("🥦".to_string()),
            s if s.contains("fruit") => Some("🍎".to_string()),
            s if s.contains("meat") => Some("🥩".to_string()),
            s if s.contains("fish") => Some("🐟".to_string()),
            s if s.contains("bread") || s.contains("wheat") => Some("🍞".to_string()),
            s if s.contains("spice") => Some("🌶️".to_string()),
            _ => None,
        }
    }
}

impl FoodStatsRepository for PostgresFoodStatsRepository {
    async fn get_overview_stats(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
    ) -> Result<OverviewStats, CoreError> {
        // Calculate accuracy_level using SQL
        let accuracy_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            WITH reaction_items AS (
              SELECT
                fr.id as reaction_id,
                fai.risk_band,
                fr.feeling
              FROM food_reactions fr
              INNER JOIN food_analysis_items fai ON fr.analysis_item_id = fai.id
              WHERE fr.realm_id = $1
                AND fr.user_id = $2
                AND fr.analysis_item_id IS NOT NULL
            ),
            accuracy_calc AS (
              SELECT
                COUNT(*) as total_count,
                SUM(
                  CASE
                    WHEN (risk_band IN ('HIGH', 'MODERATE') AND feeling IN ('MILD_ISSUES', 'BAD'))
                      OR (risk_band = 'SAFE' AND feeling IN ('GREAT', 'OKAY'))
                    THEN 1
                    ELSE 0
                  END
                ) as matched_count
              FROM reaction_items
            )
            SELECT
              CASE
                WHEN total_count = 0 THEN 0
                ELSE ROUND((matched_count::numeric / total_count::numeric) * 100)::int
              END as accuracy_level
            FROM accuracy_calc
            "#,
            [realm_id.into(), user_id.into()],
        );

        let accuracy_result = self.db.query_one(accuracy_stmt).await.map_err(|e| {
            error!("Failed to calculate accuracy level: {}", e);
            CoreError::InternalServerError
        })?;

        let accuracy_level = accuracy_result
            .and_then(|row| row.try_get::<i32>("", "accuracy_level").ok())
            .unwrap_or(0);

        // Get tracked reactions count
        let tracked_reactions_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT COUNT(*) as count
            FROM food_reactions
            WHERE realm_id = $1 AND user_id = $2
            "#,
            [realm_id.into(), user_id.into()],
        );

        let tracked_reactions_result =
            self.db
                .query_one(tracked_reactions_stmt)
                .await
                .map_err(|e| {
                    error!("Failed to get tracked reactions count: {}", e);
                    CoreError::InternalServerError
                })?;

        let tracked_reactions = tracked_reactions_result
            .and_then(|row| row.try_get::<i64>("", "count").ok())
            .unwrap_or(0);

        // Get triggered foods count (reactions with MILD_ISSUES or BAD feeling)
        let triggered_foods_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT COUNT(DISTINCT analysis_item_id) as count
            FROM food_reactions
            WHERE realm_id = $1 
              AND user_id = $2
              AND analysis_item_id IS NOT NULL
              AND feeling IN ('MILD_ISSUES', 'BAD')
            "#,
            [realm_id.into(), user_id.into()],
        );

        let triggered_foods_result =
            self.db.query_one(triggered_foods_stmt).await.map_err(|e| {
                error!("Failed to get triggered foods count: {}", e);
                CoreError::InternalServerError
            })?;

        let triggered_foods = triggered_foods_result
            .and_then(|row| row.try_get::<i64>("", "count").ok())
            .unwrap_or(0);

        // Get trigger category stats
        let trigger_stats_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT
              fat.trigger_category,
              COUNT(DISTINCT fr.id) FILTER (WHERE fr.feeling IN ('MILD_ISSUES', 'BAD')) as issue_count,
              COUNT(DISTINCT fr.id) as total_exposures
            FROM food_analysis_triggers fat
            INNER JOIN food_analysis_items fai ON fat.item_id = fai.id
            INNER JOIN food_reactions fr ON fr.analysis_item_id = fai.id
            WHERE fat.realm_id = $1
              AND fr.user_id = $2
              AND fr.analysis_item_id IS NOT NULL
            GROUP BY fat.trigger_category
            HAVING COUNT(DISTINCT fr.id) > 0
            ORDER BY issue_count DESC, total_exposures DESC
            LIMIT 10
            "#,
            [realm_id.into(), user_id.into()],
        );

        let trigger_stats_rows = self.db.query_all(trigger_stats_stmt).await.map_err(|e| {
            error!("Failed to get trigger stats: {}", e);
            CoreError::InternalServerError
        })?;

        let triggers: Vec<TriggerStats> = trigger_stats_rows
            .into_iter()
            .filter_map(|row| {
                let trigger_category: String = row.try_get("", "trigger_category").ok()?;
                let issue_count: i64 = row.try_get("", "issue_count").ok()?;
                let total_exposures: i64 = row.try_get("", "total_exposures").ok()?;
                let risk_percent = if total_exposures > 0 {
                    ((issue_count as f64 / total_exposures as f64) * 100.0).round() as i32
                } else {
                    0
                };

                Some(TriggerStats {
                    trigger_category: trigger_category.clone(),
                    emoji: Self::get_emoji_for_category(&trigger_category),
                    issue_count,
                    total_exposures,
                    risk_percent,
                })
            })
            .collect();

        // Get safe foods stats (categories with only GREAT or OKAY reactions)
        let safe_foods_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            SELECT
              fat.trigger_category,
              COUNT(DISTINCT fr.id) as safe_exposures
            FROM food_analysis_triggers fat
            INNER JOIN food_analysis_items fai ON fat.item_id = fai.id
            INNER JOIN food_reactions fr ON fr.analysis_item_id = fai.id
            WHERE fat.realm_id = $1
              AND fr.user_id = $2
              AND fr.analysis_item_id IS NOT NULL
              AND fr.feeling IN ('GREAT', 'OKAY')
            GROUP BY fat.trigger_category
            HAVING COUNT(DISTINCT fr.id) FILTER (WHERE fr.feeling IN ('MILD_ISSUES', 'BAD')) = 0
            ORDER BY safe_exposures DESC
            LIMIT 10
            "#,
            [realm_id.into(), user_id.into()],
        );

        let safe_foods_rows = self.db.query_all(safe_foods_stmt).await.map_err(|e| {
            error!("Failed to get safe foods stats: {}", e);
            CoreError::InternalServerError
        })?;

        let safe_foods: Vec<SafeFoodStats> = safe_foods_rows
            .into_iter()
            .filter_map(|row| {
                let trigger_category: String = row.try_get("", "trigger_category").ok()?;
                let safe_exposures: i64 = row.try_get("", "safe_exposures").ok()?;

                Some(SafeFoodStats {
                    trigger_category: trigger_category.clone(),
                    emoji: Self::get_emoji_for_category(&trigger_category),
                    safe_exposures,
                })
            })
            .collect();

        // Calculate meals_to_target
        // Simple estimation: if accuracy_level < target_accuracy, estimate based on current trend
        let target_accuracy = 85;
        let meals_to_target = if accuracy_level < target_accuracy && tracked_reactions > 0 {
            // Simple estimation: assume each new reaction improves accuracy by 1%
            // This is a simplified calculation
            let needed_improvement = target_accuracy - accuracy_level;
            (needed_improvement as f64 / 1.0).ceil() as i32
        } else {
            0
        };

        Ok(OverviewStats {
            accuracy_level,
            target_accuracy,
            meals_to_target,
            tracked_reactions,
            triggered_foods,
            triggers,
            safe_foods,
        })
    }

    async fn get_symptom_stats(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetSymptomStatsFilter,
    ) -> Result<SymptomStatsResponse, CoreError> {
        // Build WHERE clause
        let mut conditions = vec![
            "fr.realm_id = $1".to_string(),
            "fr.user_id = $2".to_string(),
        ];
        let mut params: Vec<sea_orm::Value> = vec![realm_id.into(), user_id.into()];
        let mut param_index = 3;

        if let Some(start_date) = filter.start_date {
            conditions.push(format!("fr.eaten_at >= ${}", param_index));
            params.push(start_date.fixed_offset().into());
            param_index += 1;
        }

        if let Some(end_date) = filter.end_date {
            conditions.push(format!("fr.eaten_at <= ${}", param_index));
            params.push(end_date.fixed_offset().into());
            param_index += 1;
        }

        if let Some(ref symptom_code) = filter.symptom_code {
            conditions.push(format!("frs.symptom_code = ${}", param_index));
            params.push(symptom_code.clone().into());
            param_index += 1;
        }

        if let Some(ref symptom_codes) = filter.symptom_code_in
            && !symptom_codes.is_empty()
        {
            let placeholders: Vec<String> = (0..symptom_codes.len())
                .map(|i| format!("${}", param_index + i))
                .collect();
            conditions.push(format!("frs.symptom_code IN ({})", placeholders.join(", ")));
            for code in symptom_codes {
                params.push(code.clone().into());
            }
            // param_index is not used after this, but we keep it for consistency
            let _ = param_index + symptom_codes.len();
        }

        let where_clause = conditions.join(" AND ");

        // Get total reactions count
        let total_reactions_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            format!(
                r#"
                SELECT COUNT(DISTINCT fr.id) as count
                FROM food_reactions fr
                LEFT JOIN food_reaction_symptoms frs ON fr.id = frs.reaction_id
                WHERE {}
                "#,
                where_clause
            ),
            params.clone(),
        );

        let total_reactions_result =
            self.db.query_one(total_reactions_stmt).await.map_err(|e| {
                error!("Failed to get total reactions count: {}", e);
                CoreError::InternalServerError
            })?;

        let total_reactions = total_reactions_result
            .and_then(|row| row.try_get::<i64>("", "count").ok())
            .unwrap_or(0);

        // Get symptom stats
        let mut order_by = "COUNT(*) DESC".to_string();
        if let Some(ref sort_str) = filter.sort {
            if let Some(field) = sort_str.strip_prefix('-') {
                match field {
                    "count" => order_by = "COUNT(*) DESC".to_string(),
                    "percentage" => order_by = "percentage DESC".to_string(),
                    "symptom_code" => order_by = "frs.symptom_code DESC".to_string(),
                    _ => {}
                }
            } else {
                match sort_str.as_str() {
                    "count" => order_by = "COUNT(*) ASC".to_string(),
                    "percentage" => order_by = "percentage ASC".to_string(),
                    "symptom_code" => order_by = "frs.symptom_code ASC".to_string(),
                    _ => {}
                }
            }
        }

        let symptom_stats_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            format!(
                r#"
                SELECT
                  frs.symptom_code,
                  COUNT(*) as count,
                  CASE
                    WHEN {} > 0 THEN ROUND((COUNT(*)::numeric / {}::numeric) * 100, 1)
                    ELSE 0
                  END as percentage
                FROM food_reaction_symptoms frs
                INNER JOIN food_reactions fr ON frs.reaction_id = fr.id
                WHERE {}
                  AND frs.symptom_code IS NOT NULL
                GROUP BY frs.symptom_code
                ORDER BY {}
                "#,
                total_reactions, total_reactions, where_clause, order_by
            ),
            params,
        );

        let symptom_stats_rows = self.db.query_all(symptom_stats_stmt).await.map_err(|e| {
            error!("Failed to get symptom stats: {}", e);
            CoreError::InternalServerError
        })?;

        let items: Vec<SymptomStats> = symptom_stats_rows
            .into_iter()
            .filter_map(|row| {
                let symptom_code: String = row.try_get("", "symptom_code").ok()?;
                let count: i64 = row.try_get("", "count").ok()?;
                let percentage: f64 = row.try_get("", "percentage").ok()?;

                Some(SymptomStats {
                    symptom_code,
                    count,
                    percentage,
                })
            })
            .collect();

        Ok(SymptomStatsResponse {
            items,
            total_reactions,
        })
    }

    async fn get_timeline_stats(
        &self,
        realm_id: Uuid,
        user_id: Uuid,
        filter: GetTimelineStatsFilter,
    ) -> Result<TimelineStatsResponse, CoreError> {
        // Determine date grouping based on granularity
        let date_format = match filter.granularity.as_str() {
            "week" => "DATE_TRUNC('week', fr.eaten_at)::date",
            "month" => "DATE_TRUNC('month', fr.eaten_at)::date",
            _ => "fr.eaten_at::date", // default to day
        };

        // Build WHERE clause
        let mut conditions = vec![
            "fr.realm_id = $1".to_string(),
            "fr.user_id = $2".to_string(),
            "fr.eaten_at >= $3".to_string(),
            "fr.eaten_at <= $4".to_string(),
        ];
        let mut params: Vec<sea_orm::Value> = vec![
            realm_id.into(),
            user_id.into(),
            filter.start_date.fixed_offset().into(),
            filter.end_date.fixed_offset().into(),
        ];

        if let Some(ref feelings) = filter.feeling_in
            && !feelings.is_empty()
        {
            let param_index = 5;
            let placeholders: Vec<String> = (0..feelings.len())
                .map(|i| format!("${}", param_index + i))
                .collect();
            conditions.push(format!("fr.feeling IN ({})", placeholders.join(", ")));
            for feeling in feelings {
                params.push(feeling.clone().into());
            }
        }

        let where_clause = conditions.join(" AND ");

        // Build ORDER BY
        let mut order_by = "date ASC".to_string();
        if let Some(ref sort_str) = filter.sort {
            if let Some(field) = sort_str.strip_prefix('-') {
                match field {
                    "date" => order_by = "date DESC".to_string(),
                    "total_reactions" => order_by = "total_reactions DESC".to_string(),
                    "positive_reactions" => order_by = "positive_reactions DESC".to_string(),
                    "negative_reactions" => order_by = "negative_reactions DESC".to_string(),
                    _ => {}
                }
            } else {
                match sort_str.as_str() {
                    "date" => order_by = "date ASC".to_string(),
                    "total_reactions" => order_by = "total_reactions ASC".to_string(),
                    "positive_reactions" => order_by = "positive_reactions ASC".to_string(),
                    "negative_reactions" => order_by = "negative_reactions ASC".to_string(),
                    _ => {}
                }
            }
        }

        let timeline_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            format!(
                r#"
                SELECT
                  {} as date,
                  COUNT(*) as total_reactions,
                  COUNT(*) FILTER (WHERE fr.feeling IN ('GREAT', 'OKAY')) as positive_reactions,
                  COUNT(*) FILTER (WHERE fr.feeling IN ('MILD_ISSUES', 'BAD')) as negative_reactions
                FROM food_reactions fr
                WHERE {}
                GROUP BY {}
                ORDER BY {}
                "#,
                date_format, where_clause, date_format, order_by
            ),
            params,
        );

        let timeline_rows = self.db.query_all(timeline_stmt).await.map_err(|e| {
            error!("Failed to get timeline stats: {}", e);
            CoreError::InternalServerError
        })?;

        let items: Vec<TimelineStats> = timeline_rows
            .into_iter()
            .filter_map(|row| {
                let date: chrono::NaiveDate = row.try_get("", "date").ok()?;
                let total_reactions: i64 = row.try_get("", "total_reactions").ok()?;
                let positive_reactions: i64 = row.try_get("", "positive_reactions").ok()?;
                let negative_reactions: i64 = row.try_get("", "negative_reactions").ok()?;

                Some(TimelineStats {
                    date: date.format("%Y-%m-%d").to_string(),
                    total_reactions,
                    positive_reactions,
                    negative_reactions,
                })
            })
            .collect();

        Ok(TimelineStatsResponse {
            items,
            start_date: filter.start_date,
            end_date: filter.end_date,
        })
    }
}
