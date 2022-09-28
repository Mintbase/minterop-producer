#!/usr/bin/env bash

network="$1"
commit_hash="$2"

# construct environment
# TODO: converge "$RUST_LOG" so we can simply pass .env directly (note: could
# escapes commas: https://cloud.google.com/run/docs/configuring/environment-variables#escaping)
source "$network.env" || exit 1
ENV="RUST_LOG=minterop-rpc=debug"
ENV+=",NEAR_RPC_URL=$NEAR_RPC_URL"
ENV+=",POSTGRES=$POSTGRES"
ENV+=",MINTBASE_ROOT=$MINTBASE_ROOT"

# deploy
gcloud run deploy "minterop-rpc-$network" \
  --image "gcr.io/omni-cloud-1/minterop-rpc:$commit_hash" \
  --set-env-vars "$ENV" \
  --service-account minterop-rpc@omni-cloud-1.iam.gserviceaccount.com \
  --region europe-west1 \
  --allow-unauthenticated
