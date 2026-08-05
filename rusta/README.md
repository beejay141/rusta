# rusta

A clean-architecture Rust API framework with dependency injection, middleware, and proc-macro support.

Built on top of [Axum](https://github.com/tokio-rs/axum), `rusta` provides:

- **Controller routing** via `#[controller]` and `#[get]`/`#[post]`/`#[put]`/`#[delete]`/`#[patch]` attributes
- **Dependency injection** with a built-in container (`rusta-di`)
- **Middleware chain** with per-handler middleware support
- **Structured logging** and **APM** integration (`rusta-logger`, `rusta-apm`)

## Quick Start

```rust
use rusta::App;
use rusta_di::Container;

#[tokio::main]
async fn main() {
    let mut container = Container::new();
    // Register your services here

    App::new()
        .container(container)
        .run("0.0.0.0:3000", |res| match res {
            Ok(addr) => println!("Listening on {}", addr),
            Err(e) => eprintln!("Error: {}", e),
        });
}
```

## Workspace

This repository is a Cargo workspace containing:

| Crate | Description |
|-------|-------------|
| `rusta` | Core framework |
| `rusta-di` | Dependency injection container |
| `rusta-di-macros` | Proc-macros for DI and routing |
| `rusta-apm` | Application performance monitoring |
| `rusta-logger` | Structured logging middleware |
| `rusta-example` | Reference blog API |

## License

Dual-licensed under MIT or Apache-2.0.
