#!/bin/sh

# FIXME: reactivate
# (
#   cd seed-testnet || exit 1
#   npm ci || exit 1
#   npx ts-node src/index.ts || exit 1
# ) || exit 1

docker-compose build || exit 1
docker-compose run tests || exit 1
docker-compose down
