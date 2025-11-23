-- Add up migration script here

-- Create food_analysis_requests table
CREATE TABLE food_analysis_requests (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL,
    prompt_id UUID NOT NULL,
    input_type VARCHAR(10) NOT NULL,
    input_content TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_realm
        FOREIGN KEY (realm_id)
        REFERENCES realms (id)
        ON DELETE CASCADE,

    CONSTRAINT fk_prompt
        FOREIGN KEY (prompt_id)
        REFERENCES prompts (id)
        ON DELETE RESTRICT,

    CONSTRAINT fk_created_by
        FOREIGN KEY (created_by)
        REFERENCES users (id)
        ON DELETE RESTRICT,

    CONSTRAINT check_input_type
        CHECK (input_type IN ('image', 'text'))
);

-- Create food_analysis_results table
CREATE TABLE food_analysis_results (
    id UUID PRIMARY KEY,
    request_id UUID NOT NULL UNIQUE,
    dishes JSONB NOT NULL,
    raw_response TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_request
        FOREIGN KEY (request_id)
        REFERENCES food_analysis_requests (id)
        ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX idx_food_analysis_requests_realm_created
    ON food_analysis_requests(realm_id, created_at DESC);

CREATE INDEX idx_food_analysis_requests_prompt
    ON food_analysis_requests(prompt_id);

CREATE INDEX idx_food_analysis_requests_created_by
    ON food_analysis_requests(created_by);

CREATE INDEX idx_food_analysis_results_request
    ON food_analysis_results(request_id);
