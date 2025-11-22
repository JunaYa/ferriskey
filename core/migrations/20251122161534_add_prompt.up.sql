-- Add up migration script here
CREATE TABLE prompts (
  id UUID PRIMARY KEY,
  realm_id UUID NOT NULL,
  name VARCHAR(255) NOT NULL,
  description TEXT NOT NULL,
  template TEXT NOT NULL,
  version VARCHAR(50) NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT true,
  is_deleted BOOLEAN NOT NULL DEFAULT false,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  deleted_at TIMESTAMP NULL,
  created_by UUID NOT NULL,
  updated_by UUID NOT NULL,
  deleted_by UUID NULL,

  CONSTRAINT fk_realm
    FOREIGN KEY (realm_id)
    REFERENCES realms (id)
    ON DELETE CASCADE,

  CONSTRAINT fk_created_by
    FOREIGN KEY (created_by)
    REFERENCES users (id)
    ON DELETE RESTRICT,

  CONSTRAINT fk_updated_by
    FOREIGN KEY (updated_by)
    REFERENCES users (id)
    ON DELETE RESTRICT,

  CONSTRAINT fk_deleted_by
    FOREIGN KEY (deleted_by)
    REFERENCES users (id)
    ON DELETE RESTRICT
);

-- Create index for common queries
CREATE INDEX idx_prompts_realm_id ON prompts(realm_id);
CREATE INDEX idx_prompts_name ON prompts(name);
CREATE INDEX idx_prompts_is_deleted ON prompts(is_deleted);
CREATE INDEX idx_prompts_is_active ON prompts(is_active);
