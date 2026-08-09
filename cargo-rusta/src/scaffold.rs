use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Create a new Rusta project from a template.
pub fn create_project(
    name: &str,
    template: &str,
    with_docker: bool,
    with_tests: bool,
    force: bool,
) -> Result<()> {
    let target_dir = PathBuf::from(name);

    if target_dir.exists() {
        if !force {
            anyhow::bail!(
                "Directory '{}' already exists. Use --force to overwrite.",
                target_dir.display()
            );
        }
        fs::remove_dir_all(&target_dir)
            .with_context(|| format!("Failed to remove existing directory '{}'", name))?;
    }

    // Locate the templates directory relative to the executable.
    // When running via `cargo run`, CARGO_MANIFEST_DIR points to cargo-rusta/.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let template_dir = PathBuf::from(manifest_dir)
        .join("src")
        .join("templates")
        .join(template);

    if !template_dir.exists() {
        anyhow::bail!(
            "Template '{}' not found at {}",
            template,
            template_dir.display()
        );
    }

    let mut hbs = Handlebars::new();
    hbs.set_strict_mode(false);

    // Render every file in the template tree.
    for entry in WalkDir::new(&template_dir).into_iter().filter_map(|e| e.ok()) {
        let src_path = entry.path();
        let rel = src_path
            .strip_prefix(&template_dir)
            .with_context(|| format!("Failed to strip prefix from {}", src_path.display()))?;

        let rel_str = rel.to_string_lossy().to_string();

        // Skip optional files/directories based on flags.
        if !with_docker {
            // Skip Docker-related files
            if rel_str == "Dockerfile.tmpl"
                || rel_str == "docker-compose.yml.tmpl"
                || rel_str == ".env.example.tmpl"
            {
                continue;
            }
        }
        if !with_tests && rel_str.starts_with("tests") {
            continue;
        }

        // Strip the `.tmpl` suffix from the destination filename.
        let dst_rel = rel_str.strip_suffix(".tmpl").unwrap_or(&rel_str).to_string();
        let dst_path = target_dir.join(dst_rel);

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path)
                .with_context(|| format!("Failed to create directory {}", dst_path.display()))?;
            continue;
        }

        let raw = fs::read_to_string(src_path)
            .with_context(|| format!("Failed to read template file {}", src_path.display()))?;

        let rendered = if src_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e == "tmpl")
            .unwrap_or(false)
        {
            hbs.render_template(&raw, &json!({ "name": name }))?
        } else {
            raw
        };

        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directory {}", parent.display()))?;
        }

        fs::write(&dst_path, rendered)
            .with_context(|| format!("Failed to write file {}", dst_path.display()))?;
    }

    println!("✓ Created Rusta project '{}' using template '{}'", name, template);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    if with_docker {
        println!("  docker compose up -d");
    }
    println!("  cargo run");
    Ok(())
}
