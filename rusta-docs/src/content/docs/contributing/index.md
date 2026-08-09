---
title: Contributing
description: How to contribute to Rusta — workflow, testing, and documentation.
sidebar:
  order: 0
---

# Contributing to Rusta

Thanks for wanting to contribute! This document explains the expected workflow, testing, and how to add docs or examples.

## Quickstart

1. Fork the repo and open a PR against `main`.
2. Keep work on a feature branch: `git checkout -b feature/your-change`.
3. Run the workspace checks locally before opening a PR:

```bash
cargo check --workspace
cargo test --workspace
```

## Repository Layout

```
rusta/
├── rusta/              # Core framework (controllers, routing, DI)
├── rusta-di/           # Dependency injection container
├── rusta-di-macros/    # Proc-macros for #[injectable], #[controller]
├── rusta-apm/          # Application Performance Monitoring
├── rusta-logger/       # Structured logging
├── rusta-example/      # Full example app (blog API)
├── cargo-rusta/        # CLI scaffolder
├── docs/               # Reference documentation
├── rusta-docs/         # Astro documentation site
└── .github/workflows/  # CI workflows
```

## Development Workflow

### Run the example app

```bash
cargo run -p rusta-example
```

### Run tests

```bash
# All workspace tests
cargo test --workspace

# Specific crate
cargo test -p rusta

# Integration tests (require Docker)
cd rusta-example
docker build -t rusta-example:test -f Dockerfile .
cargo test --tests
```

### When you change proc-macros

Run `cargo test` in `rusta-di-macros` and rebuild dependent crates:

```bash
cargo test -p rusta-di-macros
cargo check -p rusta --tests
```

### Use `cargo-rusta` to scaffold new projects

```bash
cargo run -p cargo-rusta -- new my-test-project
```

## Documentation

### Reference docs

Docs live in `docs/` (markdown files). To edit:

- `docs/getting-started.md` — Getting started guide
- `docs/guides-*.md` — How-to guides
- `docs/reference-*.md` — API reference
- `docs/recipes.md` — Common patterns

### Documentation website

The website source lives in `rusta-docs/src/content/docs/`. To add a new page:

1. Create a new `.md` file in the appropriate subdirectory
2. Add it to the sidebar in `rusta-docs/astro.config.mjs`:

```js
{ label: "My New Page", slug: "my-new-page" }
```

3. Optionally add a section divider:

```js
{
  label: "Section Name",
  items: [
    { label: "Page 1", slug: "page-1" },
    { label: "Page 2", slug: "page-2" },
  ],
}
```

### Build the docs site locally

```bash
cd rusta-docs
npm install
npm run dev
```

## Code Style

- Follow existing project patterns
- Keep public API changes minimal and document them in `CHANGELOG.md`
- Run `cargo fmt` before committing
- Use `cargo clippy` for linting

## Tests & CI

### Local testing

- Add unit tests next to the code (`#[cfg(test)] mod tests`)
- Add integration tests under `tests/` directories
- Use the testcontainers setup for end-to-end tests

### CI

GitHub Actions runs on every push and PR. See `.github/workflows/ci.yml` for the full pipeline. The pipeline:

1. Checks the workspace compiles
2. Builds and tests the `cargo-rusta` CLI
3. Runs all workspace tests
4. Deploys docs on main branch
5. Publishes crates on tagged releases

## Issue & PR Etiquette

- Reference related issues in PR descriptions (e.g. "Fixes #123")
- Keep PRs focused — one feature or fix per PR
- Add a changelog fragment under `CHANGELOG.md` (Unreleased section)
- Be responsive to review feedback

### Commit message format

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new apm middleware
fix: handle empty body in #[post]
docs: update integration testing guide
chore: bump dependencies
```

## Release Process

1. Update `CHANGELOG.md` with the new version
2. Bump versions in all `Cargo.toml` files
3. Commit: `git commit -m "chore: release v0.1.0"`
4. Tag: `git tag v0.1.0`
5. Push: `git push origin main --tags`
6. CI will publish all crates in dependency order

## Adding a New Crate

To add a new crate to the workspace:

1. Create the crate directory: `mkdir new-crate`
2. Add `Cargo.toml` with the package metadata
3. Add `src/lib.rs` (or `src/main.rs`)
4. Add to the workspace members in `Cargo.toml`:

```toml
[workspace]
members = ["rusta", "rusta-di", ..., "new-crate"]
```

5. Add publish step in `.github/workflows/ci.yml`:

```yaml
- name: Publish `new-crate`
  run: cargo publish -p new-crate
```

## Contact

- If unsure about a breaking change or design, open an issue first to discuss
- For security issues, see `SECURITY.md` if present, or contact maintainers privately
