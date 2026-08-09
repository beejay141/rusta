# cargo-rusta

A cargo subcommand for scaffolding new Rusta API projects.

## Installation

```bash
cargo install --path .
```

## Note on Dependencies

Generated projects reference all Rusta crates (`rusta`, `rusta-di`, `rusta-apm`,
`rusta-logger`) from **crates.io** using version constraints:

```toml
[dependencies]
rusta = "0.1"
rusta-di = "0.1"
rusta-apm = "0.1"
rusta-logger = "0.1"
```

The `Dockerfile` template assumes a standalone project (no local rusta workspace),
so it only copies the project's own `Cargo.toml` and `src/` directory.

## Usage

```bash
# Create a new project with the default (minimal) template
cargo rusta new my-api

# Create a project with the full blog-api template (MongoDB + Auth)
cargo rusta new my-blog --template blog-api

# Skip Docker setup
cargo rusta new my-api --no-docker

# Skip integration tests scaffold
cargo rusta new my-api --no-tests

# Overwrite existing directory
cargo rusta new my-api --force
```

## Templates

### `default` (minimal)
A minimal API with:
- A `/health` endpoint
- APM + Logger middleware
- CORS configuration
- Docker support
- Integration test scaffold

### `blog-api`
A full blog API with:
- MongoDB persistence
- JWT authentication with Argon2
- User registration & login
- APM + Logger middleware
- Docker compose with MongoDB
- Integration tests

## Generated Files

For `default` template:

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
    ├── setup/mod.rs
    └── health_tests.rs
```

## Template Variables

Templates use Handlebars syntax. The following variable is available:

- `{{name}}` — The project name

## Adding New Templates

To add a new template:

1. Create a directory under `src/templates/<template-name>/`
2. Add files with `.tmpl` extension where you want Handlebars substitution
3. Files without `.tmpl` are copied as-is
4. Use `{{name}}` in `.tmpl` files to reference the project name

Example:
```
src/templates/my-template/
├── Cargo.toml.tmpl        # Will be rendered with {{name}}
├── src/main.rs             # Will be copied as-is
└── src/config.rs.tmpl      # Will be rendered with {{name}}
```

## License

MIT OR Apache-2.0
