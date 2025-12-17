Hoarder
=======

[![][workflow-badge]][workflow-link]

Collects your favorite media and organizes them with the hierarchical tag system.

## Production

Run the following command with api to run migration:

```console
$ hoarder migration apply
```

### Container images

- [ghcr.io/chitoku-k/hoarder/api](https://github.com/chitoku-k/hoarder/pkgs/container/hoarder%2Fapi)
- [ghcr.io/chitoku-k/hoarder/ui](https://github.com/chitoku-k/hoarder/pkgs/container/hoarder%2Fui)

```console
$ docker buildx bake
```

## Development

```console
$ yarn && yarn codegen
$ docker compose up -d --build
$ docker compose exec api hoarder migration apply
```

## Configuration

### API

#### Global Options

- `--locale` (`LOCALE`): [Unicode locale identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_locale_identifier) for collation order
- `--log-format` (`LOG_FORMAT`): Log format (`compact` or `pretty`)
- `--log-level` (`LOG_LEVEL`): Log level as in [Directives](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)

The following environment variables can be used to configure PostgreSQL connection:

- `PGHOST`: Hostname
- `PGHOSTADDR`: IP address
- `PGPORT`: Port number
- `PGUSER`: Username
- `PGPASSWORD`: Password
- `PGPASSFILE`: Path to [the password file](https://www.postgresql.org/docs/current/libpq-pgpass.html)
- `PGOPTIONS`: Command-line options
- `PGDATABASE`: Database name
- `PGSSLROOTCERT`: Path to the root CA
- `PGSSLCERT`/`PGSSLKEY`: Path to the client certificate and private key in PKCS#8 format
- `PGSSLMODE`: SSL mode
- `PGAPPNAME`: Application name

#### Serve API

```
$ hoarder [serve] [OPTIONS]
```

The following command line options (or environment variables) can be used to configure API:

- `--port` (`PORT`): Port number (**required**)
- `--media-root-dir` (`MEDIA_ROOT_DIR`): Path to the media directory (**required**)
- `--media-root-url` (`MEDIA_ROOT_URL`): Public URL for media
- `--tls-cert`/`--tls-key` (`TLS_CERT`/`TLS_KEY`): Path to TLS certificate and private key for HTTPS

#### Manage GraphQL schema

To show GraphQL schema in SDL (Schema Definition Language):

```
$ hoarder schema print
```

#### Database Migration

To execute database migration:

```
$ hoarder migration [OPTIONS] (apply | drop | list | revert)
```

### UI

#### Options

The following environment variable can be used to configure UI:

- `API_URL`: Internal URL for API (**required**)

## Testing

### API

Install [cargo-make] first in case it's not installed.

```console
$ cargo install cargo-make
```

Run all tests.

```console
$ cargo make test
```

## Credits

### Icon

Icon made by [Freepik] from [www.flaticon.com][flaticon]

[workflow-link]:    https://github.com/chitoku-k/hoarder/actions?query=branch:master
[workflow-badge]:   https://img.shields.io/github/actions/workflow/status/chitoku-k/hoarder/ci.yml?branch=master&style=flat-square
[cargo-make]:       https://github.com/sagiegurari/cargo-make
[Freepik]:          https://www.flaticon.com/authors/freepik
[flaticon]:         https://www.flaticon.com/
