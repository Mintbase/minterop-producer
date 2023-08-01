#!/usr/bin/env bash

NETWORK="$1"
COMMIT_HASH="$2"

# Contains GCP_PROJECT and RPC_URL
# TODO: create and upload to GH
source "$NETWORK.env" || exit 1

GCP_ZONE="europe-west1-b"
INDEXER_SA="indexer-sa@${GCP_PROJECT}.iam.gserviceaccount.com"
INSTANCE_NAME="indexer-vm"

STARTUP_SCRIPT="gs://cicd-$NETWORK/indexer-startup.sh"
gsutil cp "$PWD/indexer-startup.sh" "$STARTUP_SCRIPT" || exit 1

gcloud compute instances delete "$INSTANCE_NAME" -q \
  --project="$GCP_PROJECT" \
  --zone="$GCP_ZONE"

gcloud compute instances create "$INSTANCE_NAME" \
  --project="$GCP_PROJECT" \
  --zone="$GCP_ZONE" \
  --machine-type=c2-standard-4 \
  --boot-disk-size=10GB \
  --metadata startup-script-url="$STARTUP_SCRIPT,IMAGE_TAG=${COMMIT_HASH},RPC_URL=${rpc_url}" \
  --service-account="$INDEXER_SA" \
  --scopes=https://www.googleapis.com/auth/cloud-platform
