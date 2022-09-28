sudo apt update
sudo apt install -y apt-transport-https ca-certificates curl gnupg-agent software-properties-common
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
sudo add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu disco stable"
sudo apt update
sudo apt install -y docker.io
sudo apt install -y jq

# Fetch meta variables from the internal GCP metadata service
# These are passed via deploy commands that deploy the compute instance
curl "http://metadata.google.internal/computeMetadata/v1/instance/?recursive=true" -H "Metadata-Flavor: Google" >meta.json

# Create env file from secrets / metadata
cat meta.json | jq '.attributes.DOTENV' | tr '&' $'\n' | tr ';' ',' | tr -d '"' >.env
COMMIT_HASH=$(cat meta.json | jq '.attributes.COMMIT_HASH' | tr -d '"')

# Pull docker image
sudo gcloud auth configure-docker --quiet
sudo docker login gcr.io
sudo docker pull gcr.io/omni-cloud-1/minterop-indexer:$COMMIT_HASH

# Setup AWS creds (secret)
mkdir .aws
sudo gcloud secrets versions access latest --secret=AWS_INDEXER_CREDS --format='get(payload.data)' | tr '_-' '/+' | base64 -d >.aws/credentials

# spawn process to delete .env file after a minute (to avoid leaking secrets)
(
  sleep 60
  rm -r .env meta.json
) &

sudo docker run \
  --log-driver=gcplogs \
  -v $PWD/.env:/app/.env \
  -v $PWD/.aws:/root/.aws \
  gcr.io/omni-cloud-1/minterop-indexer:$COMMIT_HASH
