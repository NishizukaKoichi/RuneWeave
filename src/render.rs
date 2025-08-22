use anyhow::{Context, Result};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

use crate::language_pack::get_language_pack;
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

    // Create template context
    let mut tera_ctx = TeraContext::new();
    tera_ctx.insert("project", &ctx.plan.project);
    tera_ctx.insert("services", &ctx.plan.services);
    tera_ctx.insert("toolchain", &ctx.plan.toolchain);
    tera_ctx.insert("seed", &ctx.seed);

    // Generate toolchain directory
    render_toolchain_files(&ctx.out_dir, &ctx.plan)?;

    // Generate services using language packs
    for service in &ctx.plan.services {
        let language_pack = get_language_pack(&service.language);
        language_pack.register_templates(&mut tera)?;
        language_pack.render_service(service, &ctx.out_dir, &mut tera, &tera_ctx)?;
    }

    // Generate CI workflow
    register_ci_template(&mut tera)?;
    render_ci_workflow(&tera, &tera_ctx, &ctx.out_dir, &ctx.policy)?;

    // Copy schemas
    copy_schemas(&ctx.out_dir)?;

    Ok(())
}

fn render_toolchain_files(out_dir: &Path, plan: &StackPlan) -> Result<()> {
    let toolchain_dir = out_dir.join("toolchain");
    std::fs::create_dir_all(&toolchain_dir)?;

    // Generate rust-toolchain.toml if Rust is used
    if let Some(rust_toolchain) = &plan.toolchain.rust {
        let content = format!(
            r#"[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = [
{}
]
"#,
            rust_toolchain
                .targets
                .iter()
                .map(|t| format!("    \"{t}\""))
                .collect::<Vec<_>>()
                .join(",\n")
        );
        std::fs::write(toolchain_dir.join("rust-toolchain.toml"), content)?;
    }

    // Generate .node-version if Node is used
    if let Some(node_toolchain) = &plan.toolchain.node {
        std::fs::write(toolchain_dir.join(".node-version"), &node_toolchain.version)?;
    }

    // Generate .python-version if Python is used
    if let Some(python_toolchain) = &plan.toolchain.python {
        std::fs::write(
            toolchain_dir.join(".python-version"),
            &python_toolchain.version,
        )?;
    }

    // Generate go.mod if Go is used
    if let Some(go_toolchain) = &plan.toolchain.go {
        let content = format!("module {}\n\ngo {}\n", plan.project, go_toolchain.version);
        std::fs::write(toolchain_dir.join("go.mod"), content)?;
    }

    // Generate .java-version if Java is used
    if let Some(java_toolchain) = &plan.toolchain.java {
        std::fs::write(toolchain_dir.join(".java-version"), &java_toolchain.version)?;
    }

    Ok(())
}

fn register_ci_template(tera: &mut Tera) -> Result<()> {
    // Multi-language CI workflow
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
    strategy:
      matrix:
        service:
{%- for service in services %}
          - name: {{ service.name }}
            language: {{ service.language }}
{%- endfor %}
    steps:
      - uses: actions/checkout@v4
      
      # Language-specific setup
{%- if has_rust %}
      - name: Setup Rust
        if: matrix.service.language == 'rust'
        uses: dtolnay/rust-toolchain@stable
{%- endif %}
{%- if has_node %}
      - name: Setup Node.js
        if: matrix.service.language == 'node'
        uses: actions/setup-node@v4
        with:
          node-version-file: 'toolchain/.node-version'
{%- endif %}
{%- if has_python %}
      - name: Setup Python
        if: matrix.service.language == 'python'
        uses: actions/setup-python@v5
        with:
          python-version-file: 'toolchain/.python-version'
      - name: Install Poetry
        if: matrix.service.language == 'python'
        run: pip install poetry
{%- endif %}
{%- if has_go %}
      - name: Setup Go
        if: matrix.service.language == 'go'
        uses: actions/setup-go@v5
        with:
          go-version-file: 'toolchain/go.mod'
{%- endif %}
{%- if has_java %}
      - name: Setup Java
        if: matrix.service.language == 'java'
        uses: actions/setup-java@v4
        with:
          distribution: 'temurin'
          java-version-file: 'toolchain/.java-version'
{%- endif %}
      
      # Build and test
      - name: Build and Test Rust
        if: matrix.service.language == 'rust'
        working-directory: services/${{ '{{' }} matrix.service.name {{ '}}' }}
        run: |
          cargo check --locked
          cargo test --locked
          cargo clippy -- -D warnings
      
      - name: Build and Test Node
        if: matrix.service.language == 'node'
        working-directory: services/${{ '{{' }} matrix.service.name {{ '}}' }}
        run: |
          npm install --frozen-lockfile
          npm run lint
          npm test
      
      - name: Build and Test Python
        if: matrix.service.language == 'python'
        working-directory: services/${{ '{{' }} matrix.service.name {{ '}}' }}
        run: |
          poetry install
          poetry run pytest
      
      - name: Build and Test Go
        if: matrix.service.language == 'go'
        working-directory: services/${{ '{{' }} matrix.service.name {{ '}}' }}
        run: |
          go mod download
          go test ./...
      
      - name: Build and Test Java
        if: matrix.service.language == 'java'
        working-directory: services/${{ '{{' }} matrix.service.name {{ '}}' }}
        run: mvn test

{%- if sbom %}
      
      - name: Generate SBOM
        uses: anchore/sbom-action@v0
        with:
          path: services/${{ '{{' }} matrix.service.name {{ '}}' }}
          output-file: sbom-${{ '{{' }} matrix.service.name {{ '}}' }}.spdx.json
{%- endif %}

{%- if cosign %}
      
      - name: Install cosign
        uses: sigstore/cosign-installer@v3
      
      - name: Sign SBOM
        run: cosign sign-blob --yes --bundle sbom-${{ '{{' }} matrix.service.name {{ '}}' }}.spdx.json
{%- endif %}
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: build-${{ '{{' }} matrix.service.name {{ '}}' }}-${{ '{{' }} github.sha {{ '}}' }}
          path: |
{%- if sbom %}
            sbom-${{ '{{' }} matrix.service.name {{ '}}' }}.spdx.json
{%- endif %}
{%- if cosign %}
            sbom-${{ '{{' }} matrix.service.name {{ '}}' }}.spdx.json.sig
{%- endif %}
"#,
    )?;

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

    // Check which languages are used
    let empty_vec = Vec::new();
    let services = ctx
        .get("services")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_vec);

    let has_rust = services
        .iter()
        .any(|s| s.get("language").and_then(|l| l.as_str()) == Some("rust"));
    let has_node = services
        .iter()
        .any(|s| s.get("language").and_then(|l| l.as_str()) == Some("node"));
    let has_python = services
        .iter()
        .any(|s| s.get("language").and_then(|l| l.as_str()) == Some("python"));
    let has_go = services
        .iter()
        .any(|s| s.get("language").and_then(|l| l.as_str()) == Some("go"));
    let has_java = services
        .iter()
        .any(|s| s.get("language").and_then(|l| l.as_str()) == Some("java"));

    ci_ctx.insert("has_rust", &has_rust);
    ci_ctx.insert("has_node", &has_node);
    ci_ctx.insert("has_python", &has_python);
    ci_ctx.insert("has_go", &has_go);
    ci_ctx.insert("has_java", &has_java);

    let content = tera.render("ci.yml", &ci_ctx)?;
    std::fs::write(workflows_dir.join("ci.yml"), content)?;

    Ok(())
}

fn copy_schemas(out_dir: &Path) -> Result<()> {
    let schemas_dir = out_dir.join("schemas");
    std::fs::create_dir_all(&schemas_dir)?;

    // Copy the stack.schema.json
    if let Ok(schema_content) = std::fs::read_to_string("schemas/stack.schema.json") {
        std::fs::write(schemas_dir.join("stack.schema.json"), schema_content)?;
    } else {
        // Generate from types if not found
        let schema_content = serde_json::to_string_pretty(&schemars::schema_for!(StackPlan))?;
        std::fs::write(schemas_dir.join("stack.schema.json"), schema_content)?;
    }

    Ok(())
}
