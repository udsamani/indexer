FROM rust:latest AS builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release --bin indexer

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/indexer /app/indexer
CMD ["/app/indexer"]
