# Changelog

## 0.0.4 - Unreleased

### Added

- Validated names for independently managed database connection pools.
- Non-secret PostgreSQL provider metadata for Neon, Supabase, local,
  generic, and other providers.
- Connection names and providers in tracing spans and safe debug output.

### Clarified

- Provider metadata is descriptive and never changes SQL, migrations, or
  connection behavior.
- Storexa's PostgreSQL data plane remains separate from future provider
  control-plane integrations.

## 0.0.3 - 2026-08-07

### Added

- Structured health reports with server version and query latency.
- Structured migration reports with available migration count and elapsed time.
- Pool statistics and explicit connection acquisition.
- A Storexa transaction wrapper with classified commit and rollback errors.

### Changed

- `Database::close` now borrows the shared pool handle instead of consuming it.
- `Database::run_migrations` returns a `MigrationReport`.

## 0.0.2 - 2026-08-07

### Added

- Explicit environment-variable selection for application-scoped secrets.
- Direct dotenv-file reads that do not mutate the process environment.
- PostgreSQL URL validation with credential-safe errors.
- Non-secret configuration-source metadata.
- Configuration and roadmap documentation.

## 0.0.1 - 2026-08-06

### Added

- PostgreSQL configuration from environment variables and programmatic values.
- SQLx connection pooling, health checks, and transactions.
- An application-owned migration runner.
- Consistent Storexa errors and tracing instrumentation.
- PostgreSQL integration tests for direct and pooled connections.

## 0.0.0 - 2026-08-06

- Reserved the Storexa crate namespace with a hello-world library.
