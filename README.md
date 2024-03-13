Hoarder
=======

[![][workflow-badge]][workflow-link]

Collects your favorite media and organizes them with the hierarchical tag system.

## Set up

### Development

```bash
$ docker compose up -d --build
```

### Production

```bash
$ docker buildx bake -f docker-bake.hcl
```

## Configuration

### API

The following command line options (or environment variables) can be used to configure API:

- `--log-level` (`LOG_LEVEL`): Log level as in [RUST\_LOG](https://docs.rs/env_logger/latest/env_logger/)
- `--port` (`PORT`): Port number
- `--locale` (`LOCALE`): [Unicode locale identifier](https://unicode.org/reports/tr35/tr35.html#Unicode_locale_identifier) for collation order
- `--tls-cert`/`--tls-key` (`TLS_CERT`/`TLS_KEY`): Path to TLS certificate and private key for HTTPS
- `--media-root-dir` (`MEDIA_ROOT_DIR`): Path to the media directory
- `--media-root-url` (`MEDIA_ROOT_URL`): Public URL for media

The following environment variables can be used to configure PostgreSQL connection:

- `PGHOST`: Hostname
- `PGPORT`: Port number
- `PGUSER`: Username
- `PGPASSWORD`: Password
- `PGDATABASE`: Database name
- `PGSSLROOTCERT`: Path to the root CA
- `PGSSLCERT`/`PGSSLKEY`: Path to the client certificate and private key
- `PGSSLMODE`: SSL mode
- `PGAPPNAME`: Application name

## Testing

### API

Install [cargo-make] first in case it's not installed.

```bash
$ cargo install cargo-make
```

Run all tests.

```bash
$ cargo make test
```

[workflow-link]:    https://github.com/chitoku-k/hoarder/actions?query=branch:master
[workflow-badge]:   https://img.shields.io/github/actions/workflow/status/chitoku-k/hoarder/ci.yml?branch=master&style=flat-square
[cargo-make]:       https://github.com/sagiegurari/cargo-make
