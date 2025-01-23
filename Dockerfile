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

WORKDIR /app

# Define build argument for config source location (on host) with default value
ARG CONFIG_SOURCE=config.yml
# Define the destination path in container
ENV CONFIG_PATH=/app/config.yml

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy config file from specified source to the container
COPY ${CONFIG_SOURCE} ${CONFIG_PATH}

# Expose port for Prometheus metrics
EXPOSE 9100

# Copy the binary from builder
COPY --from=builder /app/target/release/rust /usr/local/bin/app

# Print config path and run app
CMD ["sh", "-c", "echo \"Using config at: ${CONFIG_PATH}\" && app --config ${CONFIG_PATH}"]