#!/bin/sh

(cd minterop-common && cargo test) || exit 1

(
  cd seed-testnet || exit 1
  npm ci || exit 1
  npx ts-node src/index.ts || exit 1
) || exit 1

docker build . -t minterop-workspace -f Dockerfile.dev
docker-compose build || exit 1
docker-compose run tests || exit 1
docker-compose down
