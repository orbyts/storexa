# Security policy

## Supported versions

Storexa is pre-1.0. Security fixes are applied to the latest development
version. Version 0.1.0 will be the first supported crates.io release.

## Reporting

Do not open a public issue containing credentials, connection strings, private
hostnames, or exploit details. Report a suspected vulnerability privately to
the repository maintainers through GitHub's private vulnerability reporting
feature.

## Credential handling

Storexa accepts database URLs from the application and keeps them only as part
of configuration needed to establish the SQLx pool. URLs are redacted from
Storexa-produced `Debug`, tracing, and error display output. Wrapped driver
errors remain available through `std::error::Error::source()` and should be
logged only under the host application's explicit diagnostic policy.

Applications are responsible for file permissions, secret-manager policy,
environment inheritance, URL rotation, and avoiding accidental logging before
values enter Storexa.

## Scope

Provider API tokens are outside Storexa's PostgreSQL data plane. Future
control-plane adapters must use separate credential types and may not place API
tokens in `DatabaseMetadata` or tracing fields.
