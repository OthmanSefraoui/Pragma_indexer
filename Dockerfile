# Builder stage
FROM rust:latest as builder

WORKDIR /usr/src/app

# Install system dependencies including protobuf-compiler
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy the Cargo files for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -f target/release/deps/starknet_indexer*

# Copy the real source code
COPY src ./src/

# Build the real application
RUN cargo build --release

# Final stage
FROM debian:latest

WORKDIR /app

# Install runtime dependencies including curl for healthcheck
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/Pragma_indexer .

# Set environment variables
ENV RUST_LOG=info
ENV REDIS_URL=redis://redis:6379
ENV APIBARA_API_KEY=""

# Expose the API port
EXPOSE 3000

# Run the application
CMD ["./Pragma_indexer"]