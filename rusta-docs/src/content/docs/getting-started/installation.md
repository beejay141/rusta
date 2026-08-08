---
title: Installation
description: Add Ravix to your Rust project and configure optional features.
sidebar:
  order: 1
---

# Installation

## Prerequisites

- Rust 1.79 or later (2021 edition)
- Cargo package manager

## Add to Cargo.toml

```toml
[dependencies]
ravix = "0.1.0"
```

For local development, use the path dependency:

```toml
[dependencies]
ravix = { path = "../ravix" }
ravix-apm = { path = "../ravix-apm" }
ravix-logger = { path = "../ravix-logger" }
```

## Optional Features

The `ravix-apm` and `ravix-logger` crates include optional `axum-middleware` feature (enabled by default):

```toml
[dependencies]
ravix-apm = { path = "../ravix-apm" }
# Without middleware (if you want custom middleware)
ravix-apm = { path = "../ravix-apm", default-features = false }
```

## Verify Installation

Create a minimal `main.rs`:

```rust
use ravix::prelude::*;

fn main() {
    println!("Ravix installed successfully!");
}
```

Build to verify:

```bash
cargo build
```

## Next Steps

- [Quick Start](/getting-started/quick-start) — Build your first API in 5 minutes
- [Project Structure](/getting-started/project-structure) — Recommended folder layout
