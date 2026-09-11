# Configuration and secrets

Storexa consumes database credentials. It does not own or require a particular
secret manager.

## Supported sources

Applications can construct `DatabaseConfig` in four ways:

1. `DatabaseConfig::from_env()` reads `STOREXA_DATABASE_URL`, then
   `DATABASE_URL`, and falls back to a local `.env` file.
2. `DatabaseConfig::from_env_var(name)` reads an application-selected variable
   such as `PHOTARA_DEV_DATABASE_URL`.
3. `DatabaseConfig::from_dotenv_var(path, name)` reads only the selected entry
   without exporting the file into the process environment.
4. `DatabaseConfig::from_url(value)` accepts a value obtained programmatically
   from any other source.

This supports Apogee, shell exports, container secrets, CI variables,
orchestrator-injected environment variables, and custom secret managers without
adding a Storexa dependency on any of them.

## Precedence

The sources above configure PostgreSQL only. SQLite uses explicit
`SqliteDatabaseConfig::from_path(path)` or `SqliteDatabaseConfig::in_memory()`;
it does not implicitly read an environment variable, create directories, or
interpret URI options embedded in a path. Hosts choose their own configuration
and resolve relative paths against the working directory before connecting.

The conventional `from_env()` lookup order is:

1. Existing `STOREXA_DATABASE_URL`
2. Existing `DATABASE_URL`
3. `STOREXA_DATABASE_URL` loaded from `.env`
4. `DATABASE_URL` loaded from `.env`

Explicit constructors have no hidden precedence rules.

## Credential boundaries

A PostgreSQL connection URL contains everything required for data-plane access:
host, port, database, role, password, and transport options. Neon, Supabase,
hosted PostgreSQL, and local PostgreSQL all enter Storexa through this contract.

Provider control-plane operations are different. Creating a Neon project or
branch requires a Neon API credential and provider-specific identifiers. Those
operations will live in optional provider tooling in a later release; they will
not be required for ordinary Storexa database access.

Storexa never includes connection URLs in `Debug`, tracing fields, or its own
error messages. Applications must apply the same rule to values they obtain
before passing them to Storexa.

## Configuration-file ownership

Storexa is a library and does not automatically read a machine-global
`$XDG_CONFIG_HOME/storexa/config.toml`. A shared implicit file would couple
otherwise independent applications and make connection precedence ambiguous.

Host applications should own their non-secret configuration. For example,
Photara may store connection names, provider labels, pool limits, and the names
of secret environment variables in `$XDG_CONFIG_HOME/photara/config.toml`.
Actual database URLs remain in Apogee, another secret manager, or the process
environment.

`$XDG_CONFIG_HOME/storexa/config.toml` is reserved for a future Storexa CLI or
for explicitly requested shared machine defaults. Its absence has no effect on
the library.

## SQLite defaults and durability

Both SQLite constructors default to one retained connection, a 30-second pool
acquisition timeout, a five-second SQLite busy timeout, foreign keys enabled,
FULL synchronous mode, and no journal-mode override. File creation is disabled
until `with_create_if_missing(true)`. Busy timeout accepts whole milliseconds
from zero through `i32::MAX`; zero means no lock waiting.

File pools may change connection counts and recycling timeouts. Memory pools
require exactly one connection and disabled idle/lifetime recycling; replacing
or closing that physical connection loses the database. Each independently
created in-memory pool has separate contents.

`with_journal_mode(Some(SqliteJournalMode::Wal))` explicitly opts into WAL for a
local file. `None` preserves existing mode. `with_synchronous` accepts SQLx's
SQLite policies; reducing FULL durability is the application's choice. SQLite
file names are redacted from Storexa diagnostics. Names are diagnostic labels,
not paths, and must not contain secrets. Raw driver error sources and direct
SQLx errors require the host's own logging policy.
