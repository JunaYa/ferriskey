-- Update food_analysis_requests table: add device_id, user_id, updated_at, updated_by
-- First add columns as nullable
ALTER TABLE food_analysis_requests
    ADD COLUMN device_id TEXT,
    ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE RESTRICT,
    ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    ADD COLUMN updated_by UUID REFERENCES users(id) ON DELETE RESTRICT;

-- Update existing rows: use created_by as user_id and updated_by, generate device_id
UPDATE food_analysis_requests
SET
    user_id = created_by,
    updated_by = created_by,
    updated_at = created_at,
    device_id = 'migrated_' || id::text
WHERE device_id IS NULL;

-- Now set NOT NULL constraints
ALTER TABLE food_analysis_requests
    ALTER COLUMN device_id SET NOT NULL,
    ALTER COLUMN user_id SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL,
    ALTER COLUMN updated_by SET NOT NULL;

-- Update food_analysis_results table: add updated_at, updated_by, created_by
-- First add columns as nullable
ALTER TABLE food_analysis_results
    ADD COLUMN updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    ADD COLUMN updated_by UUID REFERENCES users(id) ON DELETE RESTRICT,
    ADD COLUMN created_by UUID REFERENCES users(id) ON DELETE RESTRICT;

-- Update existing rows: get user_id from related request
UPDATE food_analysis_results
SET
    created_by = (SELECT created_by FROM food_analysis_requests WHERE id = food_analysis_results.request_id),
    updated_by = (SELECT created_by FROM food_analysis_requests WHERE id = food_analysis_results.request_id),
    updated_at = created_at
WHERE created_by IS NULL;

-- Now set NOT NULL constraints
ALTER TABLE food_analysis_results
    ALTER COLUMN updated_at SET NOT NULL,
    ALTER COLUMN updated_by SET NOT NULL,
    ALTER COLUMN created_by SET NOT NULL;

-- Create indexes for food_analysis_requests
CREATE INDEX idx_food_analysis_requests_user
    ON food_analysis_requests(realm_id, user_id, created_at DESC);

CREATE INDEX idx_food_analysis_requests_device
    ON food_analysis_requests(realm_id, device_id, created_at DESC);

-- Create GIN index for food_analysis_results dishes JSONB field
CREATE INDEX idx_food_analysis_results_dishes_gin
    ON food_analysis_results USING GIN (dishes);
