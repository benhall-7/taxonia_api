# 1. Cargo build
FROM rust:1.86 AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

# 2. Final image
FROM debian:bookworm-slim
WORKDIR /app

# Install Postgres libraries
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/taxonia-service /app/app

CMD ["./app"]
