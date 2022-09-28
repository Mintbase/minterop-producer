#!/usr/bin/env bash

network="$1"
commit_hash="$2"

# this is brittle in two places: but it get's smooshed into JSON, the .env
# may not contain double quotes `"`, and because we use `&` for newline
# serialization, we can not have `&` anywhere. `tr` limits us to single
# characters, but can (unlike `sed`) handle newlines.
# We cannot use commas because of the URL, so we need to `tr` them to
# semicolons, meaning we cannot use semicolons
DOTENV=$(cat "$network".env | sed -E '/^(#.*)?$/d' | tr $'\n' '&' | tr ',' ';')
STARTUP_SCRIPT="gs://indexer-startup-scripts/indexer-startup-$network.sh"
INSTANCE_NAME="interop-indexer-$network"

gsutil cp "$PWD/minterop-indexer/indexer-startup.sh" "$STARTUP_SCRIPT"

gcloud compute instances create "$INSTANCE_NAME" \
	--project=omni-cloud-1 \
	--zone=europe-west1-b \
	--machine-type=c2-standard-4 \
	--boot-disk-size=10GB \
	--metadata startup-script-url="$STARTUP_SCRIPT,DOTENV=${DOTENV},COMMIT_HASH=${commit_hash}" \
	--service-account=indexer-vm@omni-cloud-1.iam.gserviceaccount.com \
	--scopes=https://www.googleapis.com/auth/cloud-platform #&&
#gcloud compute instances tail-serial-port-output "$INSTANCE_NAME"

# To delete the instance:
# gcloud compute instances delete $INSTANCE_NAME

# Deploys only the env (no startup script)
# This is useful debugging the startup script by manually walking through the steps after SSH'ng into machine
# gcloud compute instances create $INSTANCE_NAME \
# 		--project=omni-cloud-1 \
# 		--zone=europe-west1-b \
# 		--machine-type=c2-standard-4 \
# 		--metadata S3_BUCKET_NAME=${S3_BUCKET_NAME},S3_REGION_NAME=${S3_REGION_NAME},START_BLOCK_HEIGHT=${START_BLOCK_HEIGHT},STOP_BLOCK_HEIGHT=${STOP_BLOCK_HEIGHT},RPC_URL=${RPC_URL},NEAR_ENV=testnet \
# 		--service-account=indexer-vm@omni-cloud-1.iam.gserviceaccount.com \
# 		--scopes=https://www.googleapis.com/auth/cloud-platform
