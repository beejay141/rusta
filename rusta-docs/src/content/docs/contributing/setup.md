---
title: Development Setup
description: Set up your local environment for contributing to Rusta.
sidebar:
  order: 1
---

# Development Setup

This guide walks you through setting up your local environment for contributing to Rusta.

## Prerequisites

- **Rust 1.79+** (2021 edition) — install via [rustup](https://rustup.rs)
- **Docker** — required for integration tests
- **Node.js 18+** — only needed for docs site development
- **Git** — for version control

## Clone the Repository

```bash
git clone https://github.com/beejay141/rusta.git
cd rusta
```

## Verify the Build

Run a quick check to make sure everything compiles:

```bash
cargo check --workspace
```

This compiles all crates in the workspace without running tests.

## Run the Example App

The fastest way to see Rusta in action:

```bash
# Start MongoDB
docker compose up mongo -d

# Run the example
cargo run -p rusta-example
```

The API will be available at `http://localhost:3001`.

## Run Tests

### Unit tests

```bash
cargo test --workspace --lib
```

### Integration tests

Integration tests use testcontainers and require Docker:

```bash
# Build the test image first
docker build -t rusta-example:test -f rusta-example/Dockerfile .

# Run integration tests
cargo test -p rusta-example --tests
```

### Specific test

```bash
cargo test -p rusta-example test_register_success -- --nocapture
```

## IDE Setup

### VS Code

Recommended extensions:

- `rust-lang.rust-analyzer` — Rust language server
- `tamasfe.even-better-toml` — TOML support
- `EditorConfig.EditorConfig` — EditorConfig support

### IntelliJ / CLion

- Install the Rust plugin
- Open the workspace root as a project

## Project Structure

```
rusta/
├── rusta/              # Core framework
├── rusta-di/           # Dependency injection
├── rusta-di-macros/    # Proc-macros
├── rusta-apm/          # APM
├── rusta-logger/       # Logger
├── rusta-example/      # Example blog API
├── cargo-rusta/        # CLI scaffolder
├── docs/               # Reference docs (markdown)
├── rusta-docs/         # Astro docs site
└── .github/workflows/  # CI
```

## Common Tasks

### Format code

```bash
cargo fmt --all
```

### Lint code

```bash
cargo clippy --workspace --all-targets
```

### Build release binaries

```bash
cargo build --release --workspace
```

### Build the docs site

```bash
cd rusta-docs
npm install
npm run dev
```

The site will be available at `http://localhost:4321`.

### Test the CLI scaffolder

```bash
# Build the CLI
cargo build --release -p cargo-rusta

# Scaffold a test project
cd /tmp
rm -rf test-project
../rusta/target/release/cargo-rusta new test-project --template default
cd test-project
cargo check
```

## Troubleshooting

### "linker not found" on Linux

Install build essentials:

```bash
sudo apt install build-essential pkg-config libssl-dev
```

### Docker permission errors

Add your user to the `docker` group:

```bash
sudo usermod -aG docker $USER
# Log out and back in for changes to take effect
```

### Proc-macro errors

If you see cryptic errors from `rusta-di-macros`, try:

```bash
cargo clean
cargo check --workspace
```

### Stale build artifacts

```bash
cargo clean
rm -rf target
cargo build --workspace
```

## Next Steps

- Read the [Contributing Guide](/contributing/) for workflow and PR etiquette
- Browse the [Guides](/guides/controllers/) to understand the framework
- Look at the [Examples](/examples/blog-api/) for real-world usage
