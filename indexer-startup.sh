# Install basic dependencies
sudo apt update || exit 1
sudo apt install -y \
  apt-transport-https \
  ca-certificates \
  curl \
  jq \
  gnupg-agent \
  software-properties-common \
  postgresql-client ||
  exit 1

# Install docker
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add - || exit 1
sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu disco stable" || exit 1
sudo apt update || exit 1
sudo apt install -y docker.io || exit 1

# Fetch meta variables from the internal GCP metadata service
# These are passed via deploy commands that deploy the compute instance
curl -H "Metadata-Flavor: Google" \
  "http://metadata.google.internal/computeMetadata/v1/instance/?recursive=true" \
  >meta.json || exit 1
IMAGE_TAG=$(cat meta.json | jq '.attributes.IMAGE_TAG' | tr -d '"')
RPC_URL=$(cat meta.json | jq '.attributes.RPC_URL' | tr -d '"')

# Create env file from secrets / metadata and replace start block height
sudo gcloud secrets versions access latest \
  --secret=INDEXER_ENV --format='get(payload.data)' |
  tr '_-' '/+' |
  base64 -d >.env || exit 1
(
  source .env || exit 1
  latest_block=$(psql "$POSTGRES" -c 'select synced_height from blocks;' | head -n 3 | tail -n 1 | xargs) || exit 1
  sed -i "s/START_BLOCK_HEIGHT=.*/START_BLOCK_HEIGHT=$latest_block/" .env || exit 1
) || exit 1

# Pull docker image
sudo gcloud auth configure-docker --quiet || exit 1
sudo docker login gcr.io || exit 1
sudo docker pull gcr.io/omni-cloud-1/minterop-producer:$IMAGE_TAG || exit 1

# Setup AWS creds (secret)
mkdir .aws || exit 1
sudo gcloud secrets versions access latest \
  --secret=AWS_INDEXER_CREDS --format='get(payload.data)' |
  tr '_-' '/+' | base64 -d >.aws/credentials || exit 1

# spawn process to delete .env file after a minute (to avoid leaking secrets)
(
  sleep 600
  rm -r .env
) &

sudo docker run \
  --log-driver=gcplogs \
  -v "$PWD/.env:/app/.env" \
  -v "$PWD/.aws:/root/.aws" \
  -e "RPC_URL=$RPC_URL" \
  gcr.io/omni-cloud-1/minterop-producer:$IMAGE_TAG || exit 1
