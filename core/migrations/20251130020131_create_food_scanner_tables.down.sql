-- Drop indexes
DROP INDEX IF EXISTS idx_food_reaction_symptoms_reaction;
DROP INDEX IF EXISTS idx_food_reactions_analysis_item;
DROP INDEX IF EXISTS idx_food_reactions_user_time;
DROP INDEX IF EXISTS idx_food_analysis_triggers_realm_category;
DROP INDEX IF EXISTS idx_food_analysis_triggers_item;
DROP INDEX IF EXISTS idx_food_analysis_items_realm_risk;
DROP INDEX IF EXISTS idx_food_analysis_items_request_input;
DROP INDEX IF EXISTS idx_food_analysis_items_realm_result;
DROP INDEX IF EXISTS idx_device_profiles_user;
DROP INDEX IF EXISTS idx_device_profiles_realm_device;

-- Drop tables in reverse order of dependencies
DROP TABLE IF EXISTS food_reaction_symptoms CASCADE;
DROP TABLE IF EXISTS food_reactions CASCADE;
DROP TABLE IF EXISTS food_analysis_triggers CASCADE;
DROP TABLE IF EXISTS food_analysis_items CASCADE;
DROP TABLE IF EXISTS device_profiles CASCADE;
