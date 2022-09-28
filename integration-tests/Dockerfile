ARG IMAGE
FROM ${IMAGE:-minterop-workspace}
WORKDIR /app/integration-tests
COPY . .
RUN apt-get update && apt-get install -y inetutils-ping
CMD ["cargo", "test"]