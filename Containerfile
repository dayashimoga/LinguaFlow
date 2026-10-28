# Stage 1: Builder
FROM rust:1.80-slim-bookworm AS builder

WORKDIR /usr/src/linguaflow

# Install build prerequisites
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/lingua-core/Cargo.toml ./crates/lingua-core/
COPY crates/lingua-storage/Cargo.toml ./crates/lingua-storage/
COPY crates/lingua-engine/Cargo.toml ./crates/lingua-engine/
COPY crates/lingua-server/Cargo.toml ./crates/lingua-server/
COPY crates/lingua-cli/Cargo.toml ./crates/lingua-cli/
COPY src-tauri/Cargo.toml ./src-tauri/

# Copy source code and icons
COPY crates ./crates
COPY src-tauri ./src-tauri

# Compile production binaries
RUN cargo build --release -p lingua-cli -p lingua-server

# Stage 2: Minimal Distroless / Slim Runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root system user
RUN groupadd -g 1001 linguaflow && \
    useradd -u 1001 -g linguaflow -m -s /bin/bash linguaflow

WORKDIR /app

# Copy compiled binaries from builder
COPY --from=builder /usr/src/linguaflow/target/release/lingua /usr/local/bin/lingua

# Create and set permissions for bounded persistent data storage
RUN mkdir -p /data && chown -R linguaflow:linguaflow /data /app

USER linguaflow
ENV LINGUAFLOW_DATA_DIR=/data
ENV RUST_LOG=info

EXPOSE 8080

HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

ENTRYPOINT ["/usr/local/bin/lingua", "serve", "--host", "0.0.0.0", "--port", "8080"]
