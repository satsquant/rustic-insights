# === Build Stage ===
FROM rust:1.85.0-slim AS builder

WORKDIR /usr/src/app

# Install only essential build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    pkg-config \
    libssl-dev \
    perl \
    curl \
    git \
    clang \
    llvm-dev \
    libclang-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set path for bindgen to find libclang
ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    cargo build --release

# Copy actual source and compile
COPY src ./src

RUN cargo build --release --locked

# === Runtime Stage ===
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder
COPY --from=builder /usr/src/app/target/release/rustic-insights /app/

# Use a non-root user for security
RUN groupadd -r appuser && useradd -r -g appuser appuser
USER appuser

EXPOSE 8080

CMD ["./rustic-insights"]