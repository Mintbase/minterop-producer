#!/bin/sh

(
  cd seed-testnet || exit 1
  npm ci || exit 1
  npx ts-node src/index.ts || exit 1
) || exit 1

docker-compose build || exit 1
docker-compose up -d || exit 1
echo 'Docker system running'

sleep 10
while docker ps | grep indexer >/dev/null; do
  sleep 1
done
echo 'Indexer finished'

docker compose exec tests cargo test || exit 1

docker-compose down
