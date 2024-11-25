# Use an official Rust image as the builder
FROM rust:1.82 AS builder

# Set the working directory
WORKDIR /app

COPY . .

RUN cargo build --locked --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/bluesky-scraper /app/

ENTRYPOINT ["./bluesky-scraper"]