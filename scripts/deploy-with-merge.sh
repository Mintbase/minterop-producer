#!/usr/bin/env bash

# WARNING: this is a from-scratch redeployment, that will wipe the database

network="$1"
commit_hash="$2"

MERGE_DIR='../minterop-merge'

# resolve commit hash: try from args, otherwise try .env, default to latest
COMMIT_HASH="$commit_hash"
[[ -z "$COMMIT_HASH" ]] && COMMIT_HASH=$(sed -nE 's/COMMIT_HASH=(.*)/\1/p' .env)
[[ -z "$COMMIT_HASH" ]] && COMMIT_HASH=latest

read -r -d '' HASURA_GRANTS <<EOF
grant connect on database minterop to hasura;
grant select on all tables in schema public to hasura;
grant select on all tables in schema mb_views to hasura;
grant select on all sequences in schema mb_views to hasura;
grant usage on schema mb_views to hasura;
EOF

# Stop indexing while doing migrations, otherwise good chances of DB corruption
# between migration and indexer redeployment
gcloud compute instances delete "interop-indexer-$network" -q \
  --project=omni-cloud-1 \
  --zone=europe-west1-b

(
  source "$network.env" || exit 1
  cd minterop-common || exit 1
  # revert creating views
  DATABASE_URL="$POSTGRES" diesel migration revert || exit 1
  # revert creating database itself
  DATABASE_URL="$POSTGRES" diesel migration revert || exit 1
  # create the schema
  DATABASE_URL="$POSTGRES" diesel migration run || exit 1

  # hasura project reload metadata
  cd ../hasura || exit 1
  hasura metadata apply --envfile "../$network.env"

  # grant privileges to hasura user
  psql "$POSTGRES" -c "$HASURA_GRANTS"
) || exit 1

# Data merging: takes a long time, tightly coupled to repo structure. Should
# ideally only be done manually, so this should be sufficient
(
  cd "$MERGE_DIR" || exit 1
  ./merge.sh "$network" || exit 1
) || exit 1

"scripts/deploy-rpc.sh" "$network" "$COMMIT_HASH" || exit 1
"scripts/deploy-indexer.sh" "$network" "$COMMIT_HASH" || exit 1
