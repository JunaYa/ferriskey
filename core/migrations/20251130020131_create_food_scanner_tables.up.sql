-- Create device_profiles table
CREATE TABLE device_profiles (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by UUID NULL REFERENCES users(id),
    updated_by UUID NULL REFERENCES users(id),
    CONSTRAINT uq_device_realm UNIQUE (realm_id, device_id)
);

-- Create food_analysis_items table
CREATE TABLE food_analysis_items (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    request_id UUID NOT NULL REFERENCES food_analysis_requests(id) ON DELETE CASCADE,
    result_id UUID NOT NULL REFERENCES food_analysis_results(id) ON DELETE CASCADE,
    dish_index INTEGER NOT NULL,
    input_index INTEGER NULL,
    dish_name TEXT NOT NULL,
    safety_level TEXT NOT NULL,
    risk_score INTEGER NOT NULL,
    risk_band TEXT NOT NULL,
    summary_reason TEXT NOT NULL,
    ibd_concerns TEXT[] NOT NULL,
    ibs_concerns TEXT[] NOT NULL,
    recommendations TEXT NOT NULL,
    image_object_key TEXT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT uq_item_per_result UNIQUE (result_id, dish_index),
    CONSTRAINT check_risk_score CHECK (risk_score >= 0 AND risk_score <= 100),
    CONSTRAINT check_risk_band CHECK (risk_band IN ('SAFE', 'MODERATE', 'HIGH')),
    CONSTRAINT check_safety_level CHECK (safety_level IN ('SAFE', 'CAUTION', 'UNSAFE'))
);

-- Create food_analysis_triggers table
CREATE TABLE food_analysis_triggers (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES food_analysis_items(id) ON DELETE CASCADE,
    ingredient_name TEXT NOT NULL,
    trigger_category TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    risk_reason TEXT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT check_risk_level CHECK (risk_level IN ('HIGH', 'MEDIUM', 'LOW'))
);

-- Create food_reactions table
CREATE TABLE food_reactions (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    device_id TEXT NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    analysis_item_id UUID NULL REFERENCES food_analysis_items(id) ON DELETE SET NULL,
    eaten_at TIMESTAMP WITH TIME ZONE NOT NULL,
    feeling TEXT NOT NULL,
    symptom_onset TEXT NOT NULL,
    notes TEXT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT check_feeling CHECK (feeling IN ('GREAT', 'OKAY', 'MILD_ISSUES', 'BAD')),
    CONSTRAINT check_onset CHECK (symptom_onset IN ('LT_1H', 'H1_3H', 'H3_6H', 'NEXT_DAY'))
);

-- Create food_reaction_symptoms table
CREATE TABLE food_reaction_symptoms (
    id UUID PRIMARY KEY,
    reaction_id UUID NOT NULL REFERENCES food_reactions(id) ON DELETE CASCADE,
    symptom_code TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    updated_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    CONSTRAINT uq_reaction_symptom UNIQUE (reaction_id, symptom_code),
    CONSTRAINT check_symptom_code CHECK (
        symptom_code IN (
            'BLOATING', 'PAIN', 'GAS', 'URGENCY',
            'NAUSEA', 'CRAMPING', 'OTHER'
        )
    )
);

-- Create indexes for device_profiles
CREATE INDEX idx_device_profiles_realm_device
    ON device_profiles(realm_id, device_id);

CREATE INDEX idx_device_profiles_user
    ON device_profiles(user_id);

-- Create indexes for food_analysis_items
CREATE INDEX idx_food_analysis_items_realm_result
    ON food_analysis_items(realm_id, result_id, dish_index);

CREATE INDEX idx_food_analysis_items_request_input
    ON food_analysis_items(request_id, input_index, dish_index);

CREATE INDEX idx_food_analysis_items_realm_risk
    ON food_analysis_items(realm_id, risk_band, risk_score DESC);

-- Create indexes for food_analysis_triggers
CREATE INDEX idx_food_analysis_triggers_item
    ON food_analysis_triggers(item_id);

CREATE INDEX idx_food_analysis_triggers_realm_category
    ON food_analysis_triggers(realm_id, trigger_category);

-- Create indexes for food_reactions
CREATE INDEX idx_food_reactions_user_time
    ON food_reactions(realm_id, user_id, eaten_at DESC);

CREATE INDEX idx_food_reactions_analysis_item
    ON food_reactions(analysis_item_id);

-- Create indexes for food_reaction_symptoms
CREATE INDEX idx_food_reaction_symptoms_reaction
    ON food_reaction_symptoms(reaction_id);
