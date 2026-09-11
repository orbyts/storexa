# Roadmap through 0.2.0

The `0.0.x` Git checkpoints developed and validated Storexa's first supported
API. Each stage remains recoverable by Git tag. Version 0.2.0 extends that API
with a separate SQLite capability while retaining the PostgreSQL contract.

## 0.0.2 — Configuration and secrets

- Provider-neutral credential inputs
- Explicit environment and dotenv selection
- Programmatic overrides and safe diagnostics
- No required dependency on Apogee or another secret manager

## 0.0.3 — Lifecycle, transactions, and migrations

- Connection lifecycle and timeout behavior
- Transaction commit and rollback ergonomics
- Migration validation and reporting
- Failure-path integration coverage

## 0.0.4 — Multiple connections and provider metadata

- Multiple named `Database` instances without global state
- Optional provider metadata kept outside application SQL
- Clear separation between PostgreSQL data-plane access and provider control planes

## 0.0.5 — Portability and transfer boundaries

- Test the same PostgreSQL API against Neon and local PostgreSQL
- Document Supabase and generic PostgreSQL compatibility
- Define export/import interfaces without claiming cross-provider migration is trivial

## 0.0.6 — Release candidate

- Freeze the proposed 0.1 API
- Complete examples, security review, and compatibility documentation
- Exercise Storexa from Photara as the first real consumer

## 0.1.0 — First supported release

- Configuration
- PostgreSQL connection pooling and health checks
- Transactions
- Application-owned migrations
- Consistent errors and tracing
- Documented provider-neutral integration contract

## Explicit non-goals for 0.1.0

- An ORM or generated CRUD framework
- Photara tables or domain types
- Automatic creation of cloud-provider accounts or projects
- One-command cross-provider data migration
- SQLite support

These may be explored later as separate layers once real application usage
demonstrates the correct API.

## 0.2.0 — SQLite capability

- SQLx 0.9 SQLite through concrete configuration, pool, lease, transaction,
  health and error types alongside the unchanged PostgreSQL API.
- Explicit local-file and isolated in-memory lifecycles, validated timeouts,
  foreign keys, journal and synchronous options.
- Application-owned SQL and migrations; local-file persistence and failure tests.
- Redacted SQLite diagnostics and documented network-filesystem limitations.

No ORM, application entities, cross-engine SQL translation, cloud synchronization,
network-share writer protocol, or global registry is part of this release.
Provider control planes and data-transfer tooling remain separate future work.
