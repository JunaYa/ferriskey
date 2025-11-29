-- Drop indexes
DROP INDEX IF EXISTS idx_food_analysis_results_dishes_gin;
DROP INDEX IF EXISTS idx_food_analysis_requests_device;
DROP INDEX IF EXISTS idx_food_analysis_requests_user;

-- Revert food_analysis_results table changes
ALTER TABLE food_analysis_results
    DROP COLUMN IF EXISTS created_by,
    DROP COLUMN IF EXISTS updated_by,
    DROP COLUMN IF EXISTS updated_at;

-- Revert food_analysis_requests table changes
ALTER TABLE food_analysis_requests
    DROP COLUMN IF EXISTS updated_by,
    DROP COLUMN IF EXISTS updated_at,
    DROP COLUMN IF EXISTS user_id,
    DROP COLUMN IF EXISTS device_id;
