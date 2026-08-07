# Roadmap to 0.1.0

The `0.0.x` series develops and validates Storexa's first supported API. A stage
may be combined with the next when its scope is small; every published version
must still be independently tested and recoverable by Git tag.

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
