# Security policy

## Supported versions

The latest 0.2.x release receives security fixes. The earlier 0.0.x development
checkpoints are unsupported.

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

SQLite file paths receive the same Storexa diagnostic redaction. SQLite statement
logging is disabled on Storexa-created connections; raw SQLx errors and exposed
source chains can still contain paths, SQL, or values. Diagnostic pool names are
application-supplied non-secret labels. SQLite encryption, file permissions and
network-filesystem coordination are not provided by Storexa.

Applications are responsible for file permissions, secret-manager policy,
environment inheritance, URL rotation, and avoiding accidental logging before
values enter Storexa.

## Scope

Provider API tokens are outside Storexa's PostgreSQL data plane. Future
control-plane adapters must use separate credential types and may not place API
tokens in `DatabaseMetadata` or tracing fields.
