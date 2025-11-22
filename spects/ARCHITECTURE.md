# FerrisKey Architecture Guide

## Quick Reference

This is a quick reference guide. For detailed information, see [.cursorrules](.cursorrules).

## Architecture Layers

```
API (axum) → Application → Domain → Infrastructure
```

### Domain Layer (`core/src/domain/`)
- **Entities**: Business objects (User, Realm, Role)
- **Ports**: Traits (Repository, Service, Policy)
- **Services**: Business logic implementations
- **Policies**: Authorization checks

### Application Layer (`core/src/application/`)
- Composes domain services
- Creates concrete service instances

### Infrastructure Layer (`core/src/infrastructure/`)
- Postgres repository implementations
- External service adapters

### API Layer (`api/src/application/http/`)
- HTTP handlers (Axum)
- Request/Response DTOs
- Authentication middleware

## Adding a New Feature

1. **Domain**: Create entity, ports, service, policy
2. **Infrastructure**: Implement repository
3. **Application**: Add to service composition
4. **API**: Create handler and router
5. **Frontend**: Create API hooks and pages

## Code Style

### Rust
- Modules: `snake_case`
- Types: `PascalCase`
- Functions: `snake_case`
- Always use `Result<T, E>`
- Always check permissions with `ensure_policy`

### TypeScript
- Files: `kebab-case`
- Components: `PascalCase`
- Functions: `camelCase`
- Use TanStack Query for API calls
- Use Zustand for global state

## Key Patterns

### Service Method Pattern
```rust
async fn get_user(&self, identity: Identity, input: GetUserInput) -> Result<User, CoreError> {
    // 1. Validate realm
    let realm = self.realm_repository.get_by_name(input.realm_name).await?.ok_or(CoreError::InvalidRealm)?;
    
    // 2. Check permissions
    ensure_policy(self.policy.can_view_user(identity, realm).await, "insufficient permissions")?;
    
    // 3. Execute business logic
    self.user_repository.get_by_id(input.user_id).await.map_err(|_| CoreError::InternalServerError)
}
```

### Handler Pattern
```rust
pub async fn get_user(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<UserResponse>, ApiError> {
    let user = state.service.get_user(identity, GetUserInput { user_id, realm_name }).await.map_err(ApiError::from)?;
    Ok(Response::OK(UserResponse { data: user }))
}
```

### Frontend API Hook Pattern
```typescript
export const useGetUser = ({ realm, userId }: GetUserQueryParams) => {
  return useQuery({
    ...window.tanstackApi.get('/realms/{realm_name}/users/{user_id}', {
      path: { realm_name: realm!, user_id: userId! },
    }).queryOptions,
    enabled: !!userId && !!realm,
  })
}
```

## Commands

```bash
# Rust
cargo fmt --all
cargo clippy --all -- -D warnings
cargo test

# TypeScript
cd front && npm run lint
cd front && npm run dev
```

## Important Rules

1. ✅ Always check permissions with `ensure_policy`
2. ✅ Use `Identity` for authentication context
3. ✅ Convert database entities to domain entities
4. ✅ Invalidate queries after mutations
5. ✅ Format and lint before committing
6. ❌ Don't leak infrastructure into domain
7. ❌ Don't skip permission checks
8. ❌ Don't use database entities in domain layer

