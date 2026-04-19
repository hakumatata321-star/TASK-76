FROM rust:bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates sqlite3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/fleetreserve-backend /app/fleetreserve-backend
COPY migrations/ /app/migrations/
RUN mkdir -p /data/uploads
EXPOSE 3001
CMD ["/app/fleetreserve-backend"]
