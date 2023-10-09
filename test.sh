#!/usr/bin/env bash

PG_USER=postgres
PG_PASSWORD=password
PG_DATABASE=minterop

docker kill postgres >/dev/null 2>&1
docker rm postgres >/dev/null 2>&1
docker kill minterop-producer >/dev/null 2>&1
docker rm minterop-producer >/dev/null 2>&1
docker network create minterop >/dev/null 2>&1

docker run --name postgres \
  -p 5432:5432 \
  --net minterop \
  -e "POSTGRES_USER=$PG_USER" \
  -e "POSTGRES_PASSWORD=$PG_PASSWORD" \
  -e "POSTGRES_DB=$PG_DATABASE" \
  postgres >pg.log 2>&1 &
sleep 5

PG_STRING="postgres://$PG_USER:$PG_PASSWORD@postgres:5432/$PG_DATABASE"

docker build . -t minterop-producer-debug || exit 1
docker run --name minterop-producer \
  --net minterop \
  -e "POSTGRES=$PG_STRING" \
  -e "RPC_URL=irrelevant" \
  -e "S3_BUCKET_NAME=near-lake-data-mainnet" \
  -e "S3_REGION_NAME=eu-central-1" \
  -e "START_BLOCK_HEIGHT=0" \
  -e "MINTBASE_ROOT=mintbase1.near" \
  -e "RUST_LOG=minterop=debug,near_lake_framework=debug" \
  -e "PARAS_MARKETPLACE_ID=market.paras.near" \
  -v "$HOME/.aws:/root/.aws" \
  minterop-producer-debug >run.log
