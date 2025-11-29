# MinIO File Storage Specification

## Overview

This document describes how to introduce MinIO-backed object storage and a multi-tenant file upload pipeline to FerrisKey. The scope covers backend (domain → infrastructure), API, frontend hooks, and deployment artifacts (Docker Compose, Helm) so that files can be uploaded, listed, downloaded, and deleted consistently in local and server environments.

## Goals

- Allow authenticated users to upload arbitrary binary content (up to 50 MB per object) into MinIO and track metadata in Postgres.
- Enforce tenant isolation by realm (`realm_id`) and policy checks at the domain layer.
- Provide paginated list and CRUD-style endpoints (`POST`, `GET`, `DELETE`) under `/realms/{realm_name}/files`.
- Support resumable uploads for large files via pre-signed URLs.
- Ensure parity between local Docker Compose and production (Helm) deployments.
- Emit structured logs for every MinIO interaction (request, response, error).

## Non-goals

- Virus scanning
- End-user presigned download sharing
- CDN configuration

## Domain Design

### Entity: `StoredObject`

```rust
pub struct StoredObject {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub bucket: String,
    pub object_key: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub uploaded_by: Uuid,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}
```

### Table: `stored_objects`

| Column | Type | Notes |
| --- | --- | --- |
| `id` | `uuid` | PK (v7) |
| `realm_id` | `uuid` | FK → `realms.id` |
| `bucket` | `text` | logical bucket per realm |
| `object_key` | `text` | MinIO object path |
| `original_name` | `text` | user filename |
| `mime_type` | `text` | validated MIME |
| `size_bytes` | `bigint` | ≤ 50 MB constraint |
| `checksum_sha256` | `text` | dedup/reference |
| `metadata` | `jsonb` | optional |
| `uploaded_by` | `uuid` | FK → `users.id` |
| `created_at` / `updated_at` | `timestamptz` | default now() |
| `created_by` / `updated_by` | `uuid` | nullable audit |

Indexes:
- `(realm_id, object_key)` unique
- `(created_at DESC)` for pagination

### Ports

```rust
#[cfg_attr(test, mockall::automock)]
pub trait ObjectStoragePort: Send + Sync {
    fn put_object(
        &self,
        bucket: &str,
        object_key: &str,
        payload: Bytes,
        content_type: &str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn presign_put_url(
        &self,
        bucket: &str,
        object_key: &str,
        expires_in: Duration,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn presign_get_url(
        &self,
        bucket: &str,
        object_key: &str,
        expires_in: Duration,
    ) -> impl Future<Output = Result<PresignedUrl, CoreError>> + Send;

    fn delete_object(
        &self,
        bucket: &str,
        object_key: &str,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
```

```rust
#[cfg_attr(test, mockall::automock)]
pub trait StoredObjectRepository: Send + Sync {
    fn create(&self, input: CreateStoredObject) -> impl Future<Output = Result<StoredObject, CoreError>> + Send;
    fn list(&self, filter: StoredObjectFilter, pagination: OffsetLimit) -> impl Future<Output = Result<Paginated<StoredObject>, CoreError>> + Send;
    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<StoredObject, CoreError>> + Send;
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
}
```

### Service Interface

```rust
pub struct UploadFileInput {
    pub realm_name: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub checksum_sha256: String,
    pub metadata: serde_json::Value,
    pub use_presigned: bool,
}

#[cfg_attr(test, mockall::automock)]
pub trait FileService: Send + Sync {
    async fn initiate_upload(&self, identity: Identity, input: UploadFileInput) -> Result<UploadNegotiation, CoreError>;
    async fn complete_upload(&self, identity: Identity, object_id: Uuid) -> Result<StoredObject, CoreError>;
    async fn list_files(&self, identity: Identity, filter: StoredObjectFilter, pagination: OffsetLimit) -> Result<Paginated<StoredObject>, CoreError>;
    async fn delete_file(&self, identity: Identity, object_id: Uuid) -> Result<(), CoreError>;
    async fn get_download_url(&self, identity: Identity, object_id: Uuid) -> Result<PresignedUrl, CoreError>;
}
```

Policy: extend `FilePolicy` with `can_upload`, `can_view`, `can_delete` receiving `Identity`, `Realm`.

## Application Layer

- Compose `FileServiceImpl` with `ObjectStoragePort`, `StoredObjectRepository`, `RealmRepository`, and `FilePolicy`.
- Validate `limit` (1–100) and `offset` (≥0), else `CoreError::InvalidPagination`.
- Ensure stable sort `ORDER BY created_at DESC, id DESC`.
- Use `ensure_policy` to guard every method.
- Emit structured log events before/after MinIO calls:
  - `object_storage.request` (bucket, key, operation)
  - `object_storage.response` (duration_ms, status)
  - `object_storage.error`

## API Layer

Routes under `api/src/application/http/file/`:

| Method | Path | Description |
| --- | --- | --- |
| `POST` | `/realms/{realm_name}/files/uploads` | Initiate upload, returns either direct upload token or presigned URL |
| `POST` | `/realms/{realm_name}/files/{file_id}/complete` | Mark metadata row as ready after successful object write |
| `GET` | `/realms/{realm_name}/files` | List files, query: `offset`, `limit` (default 0/20), `mime_type`, `uploaded_by`, `created_before`, `created_after` |
| `GET` | `/realms/{realm_name}/files/{file_id}/download` | Returns short-lived presigned URL |
| `DELETE` | `/realms/{realm_name}/files/{file_id}` | Soft-delete: delete object, mark record |

Response schema:

```jsonc
{
  "items": [StoredObjectResponse],
  "offset": 0,
  "limit": 20,
  "count": 57
}
```

`StoredObjectResponse` mirrors entity fields plus `download_url` when requested.

Validation:
- Reject files >50 MB (`413 Payload Too Large`).
- Only allow `mime_type` values on allow-list (configurable).
- For invalid `offset/limit`, return `400` with message.

## Infrastructure Layer

### MinIO Adapter

- New module `core/src/infrastructure/object_storage/minio.rs` implementing `ObjectStoragePort` using `aws-sdk-s3` (MinIO-compatible).
- Configuration via `ObjectStorageConfig` loaded from env: `OBJECT_STORAGE_ENDPOINT`, `OBJECT_STORAGE_REGION`, `OBJECT_STORAGE_ACCESS_KEY`, `OBJECT_STORAGE_SECRET_KEY`, `OBJECT_STORAGE_BUCKET_PREFIX`, `OBJECT_STORAGE_USE_SSL`.
- Reuse `reqwest::Client` / `aws_sdk_s3::Client` with custom endpoint.
- All requests instrumented with `tracing`.

### Repository Implementation

- Add Sea-ORM entity `stored_objects` with audit columns.
- Implement `PostgresStoredObjectRepository`.

## Frontend

- New API module `front/src/api/files.api.ts` for initiation, completion, listing, deletion, download.
- Use TanStack Query and `window.tanstackApi`.
- Upload flow:
  1. Call `initiate_upload`.
  2. If presigned: upload via `fetch` with PUT.
  3. Call `complete_upload`.
- UI components under `front/src/pages/files/`.
- Support drag/drop and show pagination controls (offset/limit).

## Local Development (Docker Compose)

- Add `minio` service (image `minio/minio:RELEASE.2024-09-25T`), expose `9000` (API) & `9001` (console).
- Add `minio-setup` job to create default bucket (`ferriskey-local`) and access policy.
- API service env:
  - `OBJECT_STORAGE_ENDPOINT=http://localhost:9000`
  - `OBJECT_STORAGE_ACCESS_KEY=ferriskey`
  - `OBJECT_STORAGE_SECRET_KEY=ferriskeysecret`
  - `OBJECT_STORAGE_BUCKET_PREFIX=ferriskey-local`
- Frontend `.env` for console URL display (optional).

## Production Deployment (Helm)

- Extend `charts/ferriskey/values.yaml`:
  - `objectStorage.enabled`
  - `objectStorage.endpoint`
  - `objectStorage.region`
  - `objectStorage.accessKey`
  - `objectStorage.secretKey`
  - `objectStorage.bucketPrefix`
  - `objectStorage.tls`
- Add optional `minio` subchart (disabled by default). Document expectation to point at managed S3/MinIO cluster.
- Inject secrets via `charts/ferriskey/templates/secret.yaml`.
- Mount env variables into `api` and `operator` deployments.
- NetworkPolicy to allow API pods to reach MinIO endpoint.

## Observability

- Extend tracing span `object_storage` around every MinIO request.
- Metrics:
  - Counter `object_storage_requests_total{operation}`.
  - Histogram `object_storage_request_duration_ms{operation}`.
  - Gauge `object_storage_inflight`.
- Log error bodies at debug level with redaction for secrets.

## Testing Plan

1. **Unit**: mock `ObjectStoragePort` verifying service logic and pagination errors.
2. **Integration**: spin up MinIO via docker-compose in CI (service already defined). Upload fixture file, assert metadata row.
3. **API E2E**: new tests under `api/tests/it/files.rs`.
4. **Frontend**: Vitest for hooks, Cypress for upload UI (optional).
5. **Lint**: ensure `cargo fmt`, `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`, `cargo test`, `pnpm lint`.

## Security & Compliance

- Validate MIME type vs user-supplied file signature (magic bytes) before upload completion.
- Enforce tenant/realm ownership on every operation.
- Store checksums to detect tampering.
- Presigned URLs expire within 5 minutes and are single-use (tracked via cache).
- All secrets provided via env/Secret with `readOnly: true`.

## Rollout

1. Merge schema migration & repository.
2. Land MinIO adapter.
3. Add API endpoints and service orchestration.
4. Update docker-compose and Helm.
5. Ship frontend UI.
6. Verify with `cargo test`, `cargo clippy`, `pnpm lint`, manual upload in local env.
