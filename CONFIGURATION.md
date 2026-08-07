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
