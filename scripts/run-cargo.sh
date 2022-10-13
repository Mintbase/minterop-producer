#!/usr/bin/env bash

fail() {
  echo "$2"
  exit "$1"
}

quietly() {
  "$@" >/dev/null 2>&1
}

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

RPC_URL='https://event-dispatcher-z3w7d7dnea-ew.a.run.app/publish' \
  cargo run || fail "$?" "Indexer crashed"
