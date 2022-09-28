ARG IMAGE
FROM ${IMAGE:-minterop-workspace} AS builder
RUN cargo build --release -p minterop-indexer --bin minterop_indexer

# Running the app
FROM debian:latest
RUN apt-get update && apt-get install -y libpq5 ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/minterop_indexer /usr/local/bin
CMD ["/usr/local/bin/minterop_indexer"]
