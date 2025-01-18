# Build stage
FROM rust:1.83-slim-bullseye AS builder

# Set environment variables
ENV RUST_BACKTRACE=1
ENV RUST_LOG=info

# Set the working directory
WORKDIR /app

# Install OpenSSL development packages and pkg-config
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Set the working directory
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy config file
COPY config.yml /app/config.yml

# Expose port for Prometheus metrics
EXPOSE 9100

# Copy the binary from builder
COPY --from=builder /app/target/release/rust /usr/local/bin/app

# Define the default command
CMD ["app"]