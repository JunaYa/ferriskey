-- Create stored_objects table for file storage metadata
CREATE TABLE stored_objects (
    id UUID PRIMARY KEY,
    realm_id UUID NOT NULL REFERENCES realms(id) ON DELETE CASCADE,
    bucket TEXT NOT NULL,
    object_key TEXT NOT NULL,
    original_name TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL CHECK (size_bytes >= 0 AND size_bytes <= 52428800), -- Max 50 MB
    checksum_sha256 TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    uploaded_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    updated_by UUID,
    CONSTRAINT stored_objects_realm_key_unique UNIQUE (realm_id, object_key)
);

-- Index for pagination queries
CREATE INDEX stored_objects_created_at_idx ON stored_objects(created_at DESC);

-- Index for realm-based queries
CREATE INDEX stored_objects_realm_id_idx ON stored_objects(realm_id);

-- Index for uploaded_by queries
CREATE INDEX stored_objects_uploaded_by_idx ON stored_objects(uploaded_by);
