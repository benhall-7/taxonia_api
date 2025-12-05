# ==== Build stage ====
FROM rust:1.90 AS builder

WORKDIR /api
COPY . .

# for migrations later
RUN cargo install sqlx-cli --no-default-features --features postgres,rustls
RUN cargo build --release

# ==== Runtime stage ====
FROM debian:bookworm-slim
WORKDIR /api

# Be able to verify certificates for outgoing HTTPS requests
RUN apt-get update && apt-get install -y \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 10001 appuser
USER appuser

COPY --from=builder /api/target/release/taxonia_api .
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=builder /api/migrations ./migrations

ENV RUST_LOG=info \
    APP_ENV=production \
    BIND_ADDR=0.0.0.0:8080

EXPOSE 8080
CMD ["./taxonia_api"]