FROM rust:1.71-bullseye AS builder
RUN apt-get update && apt-get install -y libpq5 ca-certificates
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo build --release

# Running the app
FROM debian:bullseye
RUN apt-get update && apt-get install -y libpq5 ca-certificates
WORKDIR /app
COPY --from=builder /app/target/release/minterop_indexer /usr/local/bin
RUN touch .env
CMD ["/usr/local/bin/minterop_indexer"]
