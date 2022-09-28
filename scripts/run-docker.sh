#!/usr/bin/env bash

fail() {
  echo "$2"
  exit "$1"
}

quietly() {
  "$@" >/dev/null 2>&1
}
docker_kill_rm() {
  quietly docker kill "$1"
  quietly docker rm "$1"
}

docker_kill_rm minterop-indexer
docker_kill_rm minterop-rpc
docker_kill_rm postgresDB
quietly docker network rm minterop
quietly docker network create minterop

docker run --name postgresDB \
  -e POSTGRES_USER="postgres" \
  -e POSTGRES_PASSWORD="password" \
  -e POSTGRES_DB="minterop" \
  --network minterop --network-alias postgres \
  -p "5432:5432" -d postgres ||
  fail "$?" "Failed to run postgres container"
export POSTGRES='postgres://postgres:password@postgres:5432/minterop'
sleep 3 # wait until postgres is ready

# Use this to generate a valid `schema.rs` and truncate the DB
(
  cd minterop-common || exit "$?"
  export DATABASE_URL='postgres://postgres:password@127.0.0.1:5432/minterop'
  diesel migration run || exit "$?"
) || fail "$?" "Failed to migrate database"

# This is reproducible, use before commit
docker build -t minterop-workspace . ||
  fail "$?" "Failed to build workspace container"
docker build -t minterop-rpc -f minterop-rpc-service/Dockerfile . ||
  fail "$?" "Failed to build RPC container"
docker build -t minterop-indexer -f minterop-indexer/Dockerfile . ||
  fail "$?" "Failed to build indexer container"

docker run --network minterop --network-alias rpc --name minterop-rpc \
  -v "$PWD/.env:/app/.env" \
  -e POSTGRES="$POSTGRES" \
  -p 3000:3000 -d minterop-rpc
docker logs -f minterop-rpc >rpc.log 2>&1 &
sleep 1

docker run --network minterop --network-alias indexer --name minterop-indexer \
  -v "$HOME/.aws:/root/.aws" \
  -v "$PWD/.env:/app/.env" \
  -e POSTGRES="$POSTGRES" \
  -e RPC_URL="http://rpc:3000" \
  minterop-indexer || fail "$?" "Indexer crashed"

docker_kill_rm minterop-rpc
