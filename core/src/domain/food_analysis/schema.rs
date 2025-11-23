use serde_json::json;

/// Returns the JSON schema for food analysis LLM responses
pub fn get_food_analysis_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "dishes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "dish_name": { "type": "string" },
                        "safety_level": {
                            "type": "string",
                            "enum": ["SAFE", "CAUTION", "UNSAFE"]
                        },
                        "reason": { "type": "string" },
                        "ibd_concerns": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "ibs_concerns": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "recommendations": { "type": "string" },
                        "ingredients": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "ingredient_name": { "type": "string" },
                                    "risk_reason": { "type": "string" }
                                },
                                "required": ["ingredient_name", "risk_reason"]
                            }
                        }
                    },
                    "required": [
                        "dish_name", "safety_level", "reason",
                        "ibd_concerns", "ibs_concerns", "recommendations", "ingredients"
                    ]
                }
            }
        },
        "required": ["dishes"]
    })
}
