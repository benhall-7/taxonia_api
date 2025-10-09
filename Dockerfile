FROM rust:1.90 AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=builder /app/target/release/taxonia-service .
COPY migrations ./migrations

EXPOSE 8080
CMD ["./taxonia_service"]