# rusta-di-macros

Proc-macro support for the [Rusta](https://github.com/beejay141/rusta) framework.

## Attributes

- `#[controller("/base")]` — Register an impl block as a controller
- `#[get("/path")]`, `#[post("/path")]`, `#[put("/path")]`, `#[delete("/path")]`, `#[patch("/path")]` — Route handlers
- `#[middleware(Guard)]` — Per-handler middleware
- `#[injectable]` — Mark a struct for dependency injection

## License

Dual-licensed under MIT or Apache-2.0.
