# ── Builder Stage ─────────────────────────────────────────────────────────────
FROM rust:1.79-slim AS builder

WORKDIR /app

# Install build dependencies and rustup prerequisites
RUN apt-get update && apt-get install -y \
    curl \
    ca-certificates \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install rustup and set the default toolchain to nightly (for edition2024 support)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY ravix/Cargo.toml ravix/
COPY ravix-apm/Cargo.toml ravix-apm/
COPY ravix-logger/Cargo.toml ravix-logger/
COPY ravix-macros/Cargo.toml ravix-macros/
COPY ravix-example/Cargo.toml ravix-example/

# Build the example binary
COPY ravix/src ravix/src
COPY ravix-apm/src ravix-apm/src
COPY ravix-logger/src ravix-logger/src
COPY ravix-macros/src ravix-macros/src
COPY ravix-example/src ravix-example/src
COPY ravix-example/benches ravix-example/benches

RUN cargo build --release -p ravix-example

# ── Runtime Stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/ravix-example /app/ravix-example

# Expose the server port
EXPOSE 3001

# Run the binary
CMD ["./ravix-example"]