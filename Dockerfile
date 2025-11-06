# Build stage
FROM rust:bookworm AS builder

WORKDIR /build

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS (if needed in future)
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 yamllint && \
    mkdir -p /work && \
    chown -R yamllint:yamllint /work

# Copy binary from builder
COPY --from=builder /build/target/release/yamllint-rs /usr/local/bin/yamllint-rs

# Set permissions
RUN chmod +x /usr/local/bin/yamllint-rs

# Set working directory
WORKDIR /work

# Switch to non-root user
USER yamllint

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/yamllint-rs"]

