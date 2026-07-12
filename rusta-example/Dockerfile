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
COPY rusta/Cargo.toml rusta/
COPY rusta-apm/Cargo.toml rusta-apm/
COPY rusta-logger/Cargo.toml rusta-logger/
COPY rusta-macros/Cargo.toml rusta-macros/
COPY rusta-example/Cargo.toml rusta-example/

# Build the example binary
COPY rusta/src rusta/src
COPY rusta-apm/src rusta-apm/src
COPY rusta-logger/src rusta-logger/src
COPY rusta-macros/src rusta-macros/src
COPY rusta-example/src rusta-example/src
COPY rusta-example/benches rusta-example/benches

RUN cargo build --release -p rusta-example

# ── Runtime Stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/rusta-example /app/rusta-example

# Expose the server port
EXPOSE 3001

# Run the binary
CMD ["./rusta-example"]