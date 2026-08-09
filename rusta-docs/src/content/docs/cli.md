---
title: CLI
description: Scaffold new Rusta API projects using the cargo-rusta CLI.
sidebar:
  order: 2
---

# Cargo Rusta CLI

The `cargo-rusta` CLI is a scaffolding tool that generates new Rusta API projects from templates. It is invoked as a cargo subcommand:

```bash
cargo rusta new <NAME>
```

## Installation

Install the CLI from the workspace:

```bash
cargo install --path cargo-rusta
```

Once installed, `cargo rusta` becomes available as a cargo subcommand.

## Usage

```bash
cargo rusta new <NAME> [OPTIONS]
```

### Arguments

| Argument | Description                    |
| -------- | ------------------------------ |
| `<NAME>` | The name of the project to create |

### Options

| Option              | Description                                              | Default     |
| ------------------- | -------------------------------------------------------- | ----------- |
| `-t, --template`    | Template to use (`default`, `blog-api`)                  | `default`   |
| `--no-docker`       | Skip Docker setup (Dockerfile, docker-compose.yml, .env.example) | `false` |
| `--no-tests`        | Skip integration tests scaffold                          | `false`     |
| `--force`           | Overwrite existing directory if it exists                | `false`     |
| `-h, --help`        | Show help information                                    |             |
| `-V, --version`     | Show version information                                 |             |

## Examples

### Create a minimal API project

```bash
cargo rusta new my-api
```

This generates a minimal API with a `/health` endpoint, APM + Logger middleware, CORS configuration, and Docker support.

### Create a full blog API project

```bash
cargo rusta new my-blog --template blog-api
```

This generates a complete blog API with MongoDB persistence, JWT authentication, and integration tests.

### Skip Docker scaffolding

```bash
cargo rusta new my-api --no-docker
```

Useful for projects that don't need containerization.

### Skip integration tests

```bash
cargo rusta new my-api --no-tests
```

Useful for minimal projects or when you want to add tests manually.

### Overwrite an existing directory

```bash
cargo rusta new my-api --force
```

:::caution
`--force` **deletes** the existing directory before scaffolding. Use with care.
:::

## Templates

### `default` (minimal)

A minimal API skeleton with:

- A `/health` endpoint
- APM + Logger middleware
- CORS configuration
- Docker + docker-compose support
- Integration test scaffold

```
my-api/
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── .env.example
├── .gitignore
├── README.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── controllers/
│   │   └── mod.rs
│   └── services/
│       └── mod.rs
└── tests/integration/
    ├── mod.rs
    ├── setup/
    │   └── mod.rs
    └── health_tests.rs
```

### `blog-api`

A full blog API with:

- MongoDB persistence
- JWT authentication with Argon2 password hashing
- User registration & login
- APM + Logger middleware
- Docker Compose with MongoDB
- Integration tests

```
my-blog/
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── .env.example
├── .gitignore
├── README.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config.rs
│   ├── db.rs
│   ├── errors.rs
│   ├── middleware/
│   │   └── mod.rs
│   ├── models/
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── repositories/
│   │   ├── mod.rs
│   │   └── user_repository.rs
│   ├── services/
│   │   ├── mod.rs
│   │   └── auth_service.rs
│   └── controllers/
│       ├── mod.rs
│       └── auth_controller.rs
└── tests/integration/
    ├── mod.rs
    ├── setup/
    │   └── mod.rs
    └── auth_tests.rs
```

## After Scaffolding

After running `cargo rusta new`, follow the printed instructions:

```bash
cd my-api
docker compose up -d   # (if --no-docker was not used)
cargo run
```

The generated project is a fully working Rusta application. You can:

- Add new controllers in `src/controllers/`
- Register services in `src/services/mod.rs`
- Add integrations (database, cache, etc.) in `src/lib.rs`
- Write integration tests in `tests/integration/`

## Generated Dependencies

Generated projects reference all Rusta crates from **crates.io** using version constraints:

```toml
[dependencies]
rusta = "0.1"
rusta-di = "0.1"
rusta-apm = "0.1"
rusta-logger = "0.1"
```

The `Dockerfile` template assumes a standalone project, so it only copies the project's own `Cargo.toml` and `src/` directory.

## Adding New Templates

To add a new template to the CLI:

1. Create a directory under `src/templates/<template-name>/` in the `cargo-rusta` crate
2. Add files with `.tmpl` extension where you want Handlebars substitution
3. Files without `.tmpl` are copied as-is
4. Use `{{name}}` in `.tmpl` files to reference the project name

For example:

```
src/templates/my-template/
├── Cargo.toml.tmpl        # Will be rendered with {{name}}
├── src/main.rs             # Will be copied as-is
└── src/config.rs.tmpl      # Will be rendered with {{name}}
```

After adding the template, rebuild the CLI:

```bash
cargo build --release -p cargo-rusta
```

The new template will be available as `cargo rusta new <name> --template my-template`.

## Troubleshooting

### `Directory 'my-api' already exists`

The target directory already exists. Use `--force` to overwrite, or choose a different name.

### `Template 'foo' not found`

The template name you provided doesn't exist. Available templates are: `default`, `blog-api` (plus any custom templates you've added).

### Build errors after scaffolding

The generated project uses crates.io dependencies. If you get build errors, make sure:

1. You're online and can fetch dependencies
2. Your Rust toolchain is up to date (`rustup update`)
3. The `rusta` crates are published to crates.io (check [crates.io/crates/rusta](https://crates.io/crates/rusta))

For local development without publishing, you can temporarily edit the generated `Cargo.toml` to use path dependencies:

```toml
[dependencies]
rusta = { path = "../rusta" }
rusta-di = { path = "../rusta-di" }
rusta-apm = { path = "../rusta-apm" }
rusta-logger = { path = "../rusta-logger" }
```
