---
title: Installation
description: Add Rusta to your Rust project and configure optional features.
sidebar:
  order: 1
---

# Installation

## Prerequisites

- Rust 1.79 or later (2021 edition)
- Cargo package manager

## Scaffold a New Project

The easiest way to get started is with the `cargo-rusta` CLI, which scaffolds a complete project with all dependencies and configuration:

```bash
cargo install cargo-rusta
cargo rusta new my-api
cd my-api
cargo run
```

See the [CLI guide](/cli) for full documentation, including template options.

## Add to Existing Project

If you have an existing Rust project, add the crates you need to `Cargo.toml`:

```toml
[dependencies]
rusta = "0.1.0"
```

## Optional Features

The `rusta-apm` and `rusta-logger` crates include optional `axum-middleware` feature (enabled by default):

```toml
[dependencies]
rusta-apm = "0.1.0"
# Without middleware (if you want custom middleware)
rusta-apm = { version = "0.1.0", default-features = false }
```

## Verify Installation

Create a minimal `main.rs`:

```rust
use rusta::prelude::*;

fn main() {
    println!("Rusta installed successfully!");
}
```

Build to verify:

```bash
cargo build
```

## Next Steps

- [Quick Start](/getting-started/quick-start) — Build your first API in 5 minutes
- [Project Structure](/getting-started/project-structure) — Recommended folder layout
