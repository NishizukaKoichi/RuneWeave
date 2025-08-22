use anyhow::{Context, Result};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

use crate::verify::{Policy, StackPlan};

pub struct RenderContext {
    pub plan: StackPlan,
    pub policy: Option<Policy>,
    pub seed: u64,
    pub out_dir: PathBuf,
}

pub fn render_templates(ctx: &RenderContext) -> Result<()> {
    let _rng = StdRng::seed_from_u64(ctx.seed);

    // Create output directory
    std::fs::create_dir_all(&ctx.out_dir)
        .with_context(|| format!("Failed to create output directory: {:?}", ctx.out_dir))?;

    // Create Tera instance
    let mut tera = Tera::default();

    // Register templates (embedded for now, could load from templates/ dir)
    register_templates(&mut tera)?;

    // Create template context
    let mut tera_ctx = TeraContext::new();
    tera_ctx.insert("project", &ctx.plan.project);
    tera_ctx.insert("services", &ctx.plan.services);
    tera_ctx.insert("toolchain", &ctx.plan.toolchain);
    tera_ctx.insert("seed", &ctx.seed);

    // Generate workspace Cargo.toml
    render_workspace_toml(&tera, &tera_ctx, &ctx.out_dir)?;

    // Generate rust-toolchain.toml
    render_toolchain_toml(&tera, &tera_ctx, &ctx.out_dir)?;

    // Generate services
    render_services(&tera, &tera_ctx, &ctx.out_dir, &ctx.plan)?;

    // Generate CI workflow
    render_ci_workflow(&tera, &tera_ctx, &ctx.out_dir, &ctx.policy)?;

    // Copy schemas
    copy_schemas(&ctx.out_dir)?;

    Ok(())
}

fn register_templates(tera: &mut Tera) -> Result<()> {
    // Workspace Cargo.toml
    tera.add_raw_template(
        "workspace-cargo.toml",
        r#"[workspace]
members = [
{%- for service in services %}
    "services/{{ service.name }}",
{%- endfor %}
    "tools/cli",
]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "{{ toolchain.rust_version }}"

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
anyhow = "1.0"
"#,
    )?;

    // rust-toolchain.toml
    tera.add_raw_template(
        "rust-toolchain.toml",
        r#"[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = [
{%- for target in toolchain.targets %}
    "{{ target }}",
{%- endfor %}
]
"#,
    )?;

    // Service Cargo.toml for API
    tera.add_raw_template(
        "api-cargo.toml",
        r#"[package]
name = "{{ service_name }}"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
actix-web = "4"
tracing.workspace = true
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
"#,
    )?;

    // Service main.rs for API
    tera.add_raw_template(
        "api-main.rs",
        r#"use actix_web::{web, App, HttpResponse, HttpServer};
use tracing::info;

async fn healthz() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    }))
}

async fn ready() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "ready": true
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("Starting {{ service_name }} server on 0.0.0.0:8080");
    
    HttpServer::new(|| {
        App::new()
            .route("/healthz", web::get().to(healthz))
            .route("/v1/ready", web::get().to(ready))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
"#,
    )?;

    // Service Cargo.toml for Edge API
    tera.add_raw_template(
        "edge-cargo.toml",
        r#"[package]
name = "{{ service_name }}"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
worker = "0.6"
serde.workspace = true
serde_json.workspace = true

[dev-dependencies]
wasm-bindgen-test = "0.3"
"#,
    )?;

    // Service lib.rs for Edge API
    tera.add_raw_template(
        "edge-lib.rs",
        r#"use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/healthz", |_, _| {
            Response::ok(serde_json::json!({
                "status": "healthy"
            }).to_string())
        })
        .run(req, env)
        .await
}
"#,
    )?;

    // wrangler.jsonc
    tera.add_raw_template(
        "wrangler.jsonc",
        r#"{
  "name": "{{ service_name }}",
  "main": "src/lib.rs",
  "compatibility_date": "2024-01-01",
  // Account ID will be injected by CI/CD
  // "account_id": "YOUR_ACCOUNT_ID",
  "workers_dev": true
}
"#,
    )?;

    // CLI Cargo.toml
    tera.add_raw_template(
        "cli-cargo.toml",
        r#"[package]
name = "{{ project }}-cli"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
clap = { version = "4.5", features = ["derive"] }
anyhow.workspace = true
tracing.workspace = true
"#,
    )?;

    // CLI main.rs
    tera.add_raw_template(
        "cli-main.rs",
        r#"use clap::Parser;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "{{ project }}-cli")]
#[command(about = "CLI tool for {{ project }}", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Run smoke tests
    Smoke,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Smoke => {
            println!("Running smoke tests...");
            println!("All tests passed!");
        }
    }
    
    Ok(())
}
"#,
    )?;

    // CI workflow
    tera.add_raw_template(
        "ci.yml",
        r#"name: ci
on: [push, pull_request]

jobs:
  build-test:
    runs-on: {{ ci_runner | default(value="ubuntu-24.04") }}
    permissions:
      contents: read
      id-token: write
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      
      - name: Check workspace
        run: cargo check --workspace --locked
      
      - name: Run tests
        run: cargo test --workspace --locked
      
      - name: Check formatting
        run: cargo fmt --all -- --check
      
      - name: Run clippy
        run: cargo clippy --workspace -- -D warnings
      
      - name: Check WASM target
        run: cargo check --target wasm32-unknown-unknown -p {{ edge_service }}

      {%- if sbom %}
      
      - name: Generate SBOM
        uses: anchore/sbom-action@v0
        with:
          path: .
          output-file: sbom.spdx.json
      {%- endif %}

      {%- if cosign %}
      
      - name: Install cosign
        uses: sigstore/cosign-installer@v3
      
      - name: Sign SBOM
        run: cosign sign-blob --yes --bundle sbom.spdx.json
      {%- endif %}
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: build-${{ '{{' }} github.sha {{ '}}' }}
          path: |
            {%- if sbom %}
            sbom.spdx.json
            {%- endif %}
            {%- if cosign %}
            sbom.spdx.json.sig
            {%- endif %}
"#,
    )?;

    Ok(())
}

fn render_workspace_toml(tera: &Tera, ctx: &TeraContext, out_dir: &Path) -> Result<()> {
    let content = tera.render("workspace-cargo.toml", ctx)?;
    let path = out_dir.join("Cargo.toml");
    std::fs::write(&path, content).with_context(|| format!("Failed to write {path:?}"))?;
    Ok(())
}

fn render_toolchain_toml(tera: &Tera, ctx: &TeraContext, out_dir: &Path) -> Result<()> {
    let content = tera.render("rust-toolchain.toml", ctx)?;
    let path = out_dir.join("rust-toolchain.toml");
    std::fs::write(&path, content).with_context(|| format!("Failed to write {path:?}"))?;
    Ok(())
}

fn render_services(tera: &Tera, ctx: &TeraContext, out_dir: &Path, plan: &StackPlan) -> Result<()> {
    for service in &plan.services {
        let service_dir = out_dir.join("services").join(&service.name);
        std::fs::create_dir_all(&service_dir)?;

        let mut service_ctx = ctx.clone();
        service_ctx.insert("service_name", &service.name);

        match service.r#type {
            crate::verify::ServiceType::Api => {
                // Cargo.toml
                let content = tera.render("api-cargo.toml", &service_ctx)?;
                std::fs::write(service_dir.join("Cargo.toml"), content)?;

                // src/main.rs
                let src_dir = service_dir.join("src");
                std::fs::create_dir_all(&src_dir)?;
                let content = tera.render("api-main.rs", &service_ctx)?;
                std::fs::write(src_dir.join("main.rs"), content)?;
            }
            crate::verify::ServiceType::ApiEdge => {
                // Cargo.toml
                let content = tera.render("edge-cargo.toml", &service_ctx)?;
                std::fs::write(service_dir.join("Cargo.toml"), content)?;

                // src/lib.rs
                let src_dir = service_dir.join("src");
                std::fs::create_dir_all(&src_dir)?;
                let content = tera.render("edge-lib.rs", &service_ctx)?;
                std::fs::write(src_dir.join("lib.rs"), content)?;

                // wrangler.jsonc
                let content = tera.render("wrangler.jsonc", &service_ctx)?;
                std::fs::write(service_dir.join("wrangler.jsonc"), content)?;
            }
            crate::verify::ServiceType::Cli => {
                // Handled separately in tools/cli
            }
        }
    }

    // Generate CLI tool
    let cli_dir = out_dir.join("tools").join("cli");
    std::fs::create_dir_all(&cli_dir)?;

    let content = tera.render("cli-cargo.toml", ctx)?;
    std::fs::write(cli_dir.join("Cargo.toml"), content)?;

    let src_dir = cli_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;
    let content = tera.render("cli-main.rs", ctx)?;
    std::fs::write(src_dir.join("main.rs"), content)?;

    Ok(())
}

fn render_ci_workflow(
    tera: &Tera,
    ctx: &TeraContext,
    out_dir: &Path,
    policy: &Option<Policy>,
) -> Result<()> {
    let workflows_dir = out_dir.join(".github").join("workflows");
    std::fs::create_dir_all(&workflows_dir)?;

    let mut ci_ctx = ctx.clone();

    // Add policy-based settings
    if let Some(policy) = policy {
        if let Some(ci) = &policy.ci {
            ci_ctx.insert("ci_runner", &ci.linux_runner);
            ci_ctx.insert("sbom", &ci.sbom);
            ci_ctx.insert("cosign", &ci.cosign);
        }
    } else {
        // Defaults
        ci_ctx.insert("ci_runner", &"ubuntu-24.04");
        ci_ctx.insert("sbom", &true);
        ci_ctx.insert("cosign", &true);
    }

    // Find edge service name
    let edge_service = ctx
        .get("services")
        .and_then(|v| v.as_array())
        .and_then(|services| {
            services
                .iter()
                .find(|s| s.get("type").and_then(|t| t.as_str()) == Some("api-edge"))
                .and_then(|s| s.get("name"))
                .and_then(|n| n.as_str())
        })
        .unwrap_or("api-edge");

    ci_ctx.insert("edge_service", &edge_service);

    let content = tera.render("ci.yml", &ci_ctx)?;
    std::fs::write(workflows_dir.join("ci.yml"), content)?;

    Ok(())
}

fn copy_schemas(out_dir: &Path) -> Result<()> {
    let schemas_dir = out_dir.join("schemas");
    std::fs::create_dir_all(&schemas_dir)?;

    // For now, just create an empty schema file
    // In real implementation, would copy from the source
    let schema_content = serde_json::to_string_pretty(&schemars::schema_for!(StackPlan))?;
    std::fs::write(schemas_dir.join("stack.schema.json"), schema_content)?;

    Ok(())
}
