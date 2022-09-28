#!/usr/bin/env bash

fail() {
  echo "$2"
  exit "$1"
}

quietly() {
  "$@" >/dev/null 2>&1
}

quietly killall minterop_rpc
quietly docker kill postgresDB
quietly docker rm postgresDB

docker run --name postgresDB \
  -e POSTGRES_USER="postgres" \
  -e POSTGRES_PASSWORD="password" \
  -e POSTGRES_DB="minterop" \
  --network minterop --network-alias postgres \
  -p "5432:5432" -d postgres ||
  fail "$?" "Failed to run postgres container"
export POSTGRES='postgres://postgres:password@127.0.0.1:5432/minterop'
sleep 3 # wait until postgres is ready

# Use this to generate a valid `schema.rs` and truncate the DB
(
  cd minterop-common || exit "$?"
  export DATABASE_URL="$POSTGRES"
  diesel migration run || exit "$?"
) || fail "$?" "Failed to migrate database"

RUST_LOG='minterop-rpc=debug' cargo run -p minterop-rpc-service >rpc.log 2>&1 &
sleep 1

RPC_URL='https://event-dispatcher-z3w7d7dnea-ew.a.run.app/publish' \
  cargo run -p minterop-indexer || fail "$?" "Indexer crashed"
