# Genesis Protocol — Multi-stage Production Build
#
# Stage 1: Compile Rust binary with release optimizations
# Stage 2: Minimal runtime image (no compiler, no source)

# ── Build Stage ──────────────────────────────────────────
FROM rust:1.77-slim AS builder

WORKDIR /build

# Copy manifests first (layer caching for dependencies)
COPY Cargo.toml Cargo.lock ./
COPY crates/genesis-dna/Cargo.toml crates/genesis-dna/Cargo.toml
COPY crates/metabolism/Cargo.toml crates/metabolism/Cargo.toml
COPY crates/apostle/Cargo.toml crates/apostle/Cargo.toml
COPY crates/ecosystem/Cargo.toml crates/ecosystem/Cargo.toml
COPY crates/evolution/Cargo.toml crates/evolution/Cargo.toml
COPY crates/gateway/Cargo.toml crates/gateway/Cargo.toml

# Create dummy source files to cache dependency compilation
RUN mkdir -p src && echo "fn main() {}" > src/main.rs && \
    for crate in genesis-dna metabolism apostle ecosystem evolution gateway; do \
        mkdir -p crates/$crate/src && echo "" > crates/$crate/src/lib.rs; \
    done

RUN cargo build --release 2>/dev/null || true

# Copy real source code
COPY . .

# Touch source files to invalidate cache for our code only
RUN find . -name "*.rs" -exec touch {} +

# Build release binary
RUN cargo build --release --bin genesis-protocol

# ── Runtime Stage ────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Non-root user for security
RUN useradd --create-home --shell /bin/bash genesis
USER genesis
WORKDIR /home/genesis

# Copy binary
COPY --from=builder /build/target/release/genesis-protocol /usr/local/bin/genesis-protocol

# Data directory for world state persistence (world_state.json written to CWD)
RUN mkdir -p /home/genesis/data
VOLUME /home/genesis/data
WORKDIR /home/genesis/data

# Expose HTTP gateway port
EXPOSE 3000

# Environment defaults (override at runtime)
ENV RUST_LOG=info

# Health check — hit /status every 30s
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:3000/status || exit 1

# Run
ENTRYPOINT ["genesis-protocol"]
