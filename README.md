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
$ docker buildx bake
```

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
