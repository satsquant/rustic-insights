# Build stage
FROM rust:1.85-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY config ./config

# Build in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from build stage
COPY --from=builder /usr/src/app/target/release/rustic-insights /app/rustic-insights

# Create config directory and copy config files
COPY --from=builder /usr/src/app/config /app/config

# Set environment variables for better diagnostics
ENV RUST_BACKTRACE=1
ENV RUST_LOG=info
ENV APP__SERVER__HOST=0.0.0.0
ENV APP__SERVER__PORT=8080

# Use a non-root user for security
RUN groupadd -r appuser && useradd -r -g appuser appuser
USER appuser

# Expose the application port
EXPOSE 8080

# Run the binary
CMD ["/app/rustic-insights"]