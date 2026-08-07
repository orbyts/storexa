# Data-transfer boundaries

Moving a PostgreSQL application between providers is a workflow, not a change
to Storexa's CRUD code. The runtime repository SQL remains PostgreSQL, while
the transfer process must account for schema, data, extensions, roles,
sequences, validation, and cutover.

## Proposed boundary

Storexa may eventually coordinate these phases:

1. Accept separately configured source and target databases.
2. Verify both endpoints and record non-secret provider metadata.
3. Inspect application-owned migration state and target prerequisites.
4. Invoke an explicit transfer mechanism supplied by tooling.
5. Run application-defined verification checks.
6. Report results without retaining credentials.

The transfer mechanism is intentionally not a public Rust trait in 0.0.5. A
trait designed before Photara performs a real transfer would prematurely lock
in assumptions about subprocesses, streaming, snapshots, downtime, and cloud
APIs.

## Tool responsibilities

For an ordinary PostgreSQL-to-PostgreSQL move, `pg_dump` and `pg_restore` are
the baseline schema/data tools. Large or low-downtime migrations may require
`COPY`, logical replication, or provider tooling. These operate through direct
administrative connections and are not replacements for application-owned
migration files.

Storexa's role is coordination, safe configuration, connection verification,
consistent errors, and tracing. It should not reimplement the PostgreSQL dump
format or silently execute provider control-plane operations.

## Provider control planes

Creating projects, branches, databases, roles, or network policies requires
provider-specific credentials. Future adapters may provision a target and
return a PostgreSQL connection configuration, but those adapters remain
separate from Storexa's provider-neutral data plane.

## Secrets and recovery

Source and target URLs must come from the existing secret-input contract and
must never enter logs or transfer reports. A transfer plan must also state its
rollback point and verification criteria before cutover.
