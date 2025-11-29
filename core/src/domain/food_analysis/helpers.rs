use uuid::Uuid;

use crate::domain::food_analysis::{
    entities::{DishAnalysis, FoodAnalysisItem, FoodAnalysisTrigger},
    value_objects::{
        ingredient_to_trigger_category, map_safety_to_risk, risk_reason_to_level,
        safety_level_to_string,
    },
};

/// Create food analysis items and triggers from dishes
pub fn create_items_and_triggers_from_dishes(
    realm_id: Uuid,
    request_id: Uuid,
    result_id: Uuid,
    dishes: &[DishAnalysis],
    created_by: Uuid,
) -> (Vec<FoodAnalysisItem>, Vec<FoodAnalysisTrigger>) {
    let mut items = Vec::new();
    let mut triggers = Vec::new();

    for (dish_index, dish) in dishes.iter().enumerate() {
        // Map safety_level to risk_score and risk_band
        let (risk_score, risk_band) = map_safety_to_risk(&dish.safety_level, None);
        let safety_level_str = safety_level_to_string(&dish.safety_level);

        // Create item
        let item = FoodAnalysisItem::new(
            realm_id,
            request_id,
            result_id,
            dish_index as i32,
            None, // input_index - could be enhanced to track which input this came from
            dish.dish_name.clone(),
            safety_level_str,
            risk_score,
            risk_band,
            dish.reason.clone(),
            dish.ibd_concerns.clone(),
            dish.ibs_concerns.clone(),
            dish.recommendations.clone(),
            None, // image_object_key - could be enhanced to store dish images
            created_by,
        );

        let item_id = item.id;
        items.push(item);

        // Create triggers from ingredients
        for ingredient in &dish.ingredients {
            let trigger_category = ingredient_to_trigger_category(&ingredient.ingredient_name);
            let risk_level = risk_reason_to_level(&ingredient.risk_reason);

            let trigger = FoodAnalysisTrigger::new(
                realm_id,
                item_id,
                ingredient.ingredient_name.clone(),
                trigger_category,
                risk_level,
                Some(ingredient.risk_reason.clone()),
                created_by,
            );

            triggers.push(trigger);
        }
    }

    (items, triggers)
}
