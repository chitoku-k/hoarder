#!/usr/bin/env bash

export PGPORT=5432
export PGDATABASE=hoarder_test
export PGUSER=hoarder_test
export PGPASSWORD=hoarder_test

echo -n 'starting postgres ... ' >&2

POSTGRES_IMAGE="${POSTGRES_IMAGE:-postgres:17.5}"
POSTGRES_OPTIONS=(
    -c fsync=off
    -c full_page_writes=off
    -c max_connections=128
    -c synchronous_commit=off
)

POSTGRES_CONTAINER_ID=$(docker create \
    --env=PGDATA=/dev/shm/pgdata/data \
    --env=POSTGRES_DB="$PGDATABASE" \
    --env=POSTGRES_USER="$PGUSER" \
    --env=POSTGRES_PASSWORD="$PGPASSWORD" \
    --publish=":$PGPORT" \
    --shm-size=512m \
    --tmpfs=/var/lib/postgresql/data \
    "$POSTGRES_IMAGE" \
    "${POSTGRES_OPTIONS[@]}")

trap "docker rm --force '$POSTGRES_CONTAINER_ID' > /dev/null" EXIT
docker start "$POSTGRES_CONTAINER_ID" > /dev/null

export PGHOST=localhost
export PGPORT=$(docker port "$POSTGRES_CONTAINER_ID" $PGPORT/tcp | grep 0.0.0.0 | sed -E 's/.+://')

until docker exec "$POSTGRES_CONTAINER_ID" pg_isready --host=0.0.0.0 &> /dev/null; do
    if docker inspect --format='{{.State.Status}}' "$POSTGRES_CONTAINER_ID" | grep -q exited; then
        echo 'failed' >&2
        docker logs --timestamps "$POSTGRES_CONTAINER_ID"
        exit 1
    fi

    sleep 0.05
done

echo 'ok' >&2
