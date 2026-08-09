use clap::{Parser, Subcommand};

mod scaffold;

#[derive(Parser)]
#[command(
    name = "cargo-rusta",
    bin_name = "cargo rusta",
    version,
    about = "Scaffold new Rusta API projects"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Rusta API project
    New {
        /// Project name
        name: String,

        /// Template to use (default, blog-api)
        #[arg(short, long, default_value = "default")]
        template: String,

        /// Skip Docker setup (Dockerfile + docker-compose.yml)
        #[arg(long, default_value_t = false)]
        no_docker: bool,

        /// Skip integration tests scaffold
        #[arg(long, default_value_t = false)]
        no_tests: bool,

        /// Overwrite existing directory if it exists
        #[arg(long, default_value_t = false)]
        force: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New {
            name,
            template,
            no_docker,
            no_tests,
            force,
        } => scaffold::create_project(&name, &template, !no_docker, !no_tests, force)?,
    }
    Ok(())
}
