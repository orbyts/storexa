# Changelog

## 0.1.0 - 2026-08-07

Storexa's first supported release.

### Included

- Environment, dotenv, and programmatic PostgreSQL configuration.
- SQLx connection pooling, health diagnostics, transactions, and
  application-owned migrations.
- Named connections and provider metadata without provider-specific SQL paths.
- Consistent secret-safe errors and `tracing` instrumentation.
- Verified Neon and PostgreSQL 18 integration coverage.
- A documented public API, compatibility contract, and security policy.

## 0.0.6 - 2026-08-07

### Added

- A proposed 0.1 public API contract and application-owned repository example.
- A security policy and dependency-update monitoring.
- Crate-level enforcement for documented public APIs and no unsafe code.
- Release-candidate checks for examples, doctests, and package construction.

### Changed

- SQLx-backed error variants now have classified, redacted `Debug` output;
  deliberate callers can still inspect the source error.
- GitHub Actions now uses the current Node 24 checkout action.

## 0.0.5 - 2026-08-07

### Added

- A PostgreSQL 18 CI service that exercises Storexa against a provider-neutral
  server in addition to the manually run Neon tests.
- A compatibility matrix distinguishing verified, expected, and unsupported
  PostgreSQL connection modes.
- An explicit future data-transfer boundary for native PostgreSQL tools,
  application migrations, verification, and provider control planes.
- Guidance for application-owned XDG configuration and secret references.

### Changed

- Integration-test environment variables and connection names are now
  provider-neutral.

## 0.0.4 - 2026-08-07

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
