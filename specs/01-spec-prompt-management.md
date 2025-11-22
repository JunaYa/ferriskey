# Prompt Management Specification

## Overview

This specification describes the prompt management system for the FerrisKey project. The system allows users to manage prompts for the LLM.

## struct Prompt
// Prompt entity used for prompt management in the FerrisKey system.
// This struct represents a prompt template with versioning, audit trail, and soft-delete flags.
```rust
pub struct Prompt {
    /// Prompt unique identifier (UUID v7)
    pub id: Uuid,
    /// Foreign key, references the realm this prompt belongs to
    pub realm_id: Uuid,
    /// Human-readable prompt name
    pub name: String,
    /// Description of the prompt's purpose
    pub description: String,
    /// The template string for the prompt
    pub template: String,
    /// Version string (e.g., "v1", "2024-06-30")
    pub version: String,
    /// Whether the prompt is active and can be used
    pub is_active: bool,
    /// Whether the prompt is soft-deleted
    pub is_deleted: bool,
    /// Timestamp when the prompt was created (UTC)
    pub created_at: DateTime<Utc>,
    /// Timestamp when the prompt was last updated (UTC)
    pub updated_at: DateTime<Utc>,
    /// Timestamp when the prompt was deleted (if any)
    pub deleted_at: Option<DateTime<Utc>>,
    /// User ID who created the prompt
    pub created_by: Uuid,
    /// User ID who last updated the prompt
    pub updated_by: Uuid,
    /// User ID who deleted the prompt (if any)
    pub deleted_by: Option<Uuid>,
}
```

## Features

- Add a new prompt
- Edit an existing prompt
- Delete a prompt
- View all prompts
- View a specific prompt

## API Endpoints

### Create a New Prompt

```http
POST /prompts
Content-Type: application/json

{
    "name": "string",
    "description": "string",
    "template": "string"
}
```

### Update a Prompt

```http
PUT /prompts/{id}
```

### Delete a Prompt

```http
DELETE /prompts/{id}
```

### Get All Prompts
supports pagination and filtering
```http
GET /prompts?offset=1&limit=10&name=string&description=string
```

### Get a Specific Prompt

```http
GET /prompts/{id}
```

## in project implementation
implement the feature use ./specs/01-spec-prompt-management.md as a reference
