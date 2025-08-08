use crate::types::{Plan, Result, ServiceType};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

pub fn generate_scaffold(plan: &Plan, seed: u64, output_dir: &Path) -> Result<Vec<String>> {
    let _rng = ChaCha8Rng::seed_from_u64(seed);
    let mut files_generated = Vec::new();

    fs::create_dir_all(output_dir)?;

    let mut tera = Tera::default();
    load_templates(&mut tera)?;

    let mut context = Context::new();
    context.insert("project", &plan.project);
    context.insert("services", &plan.services);
    context.insert("tools", &plan.tools);
    context.insert("infrastructure", &plan.infrastructure);

    // Generate workspace Cargo.toml
    let workspace_content = tera.render("workspace.toml", &context)?;
    fs::write(output_dir.join("Cargo.toml"), workspace_content)?;
    files_generated.push("Cargo.toml".to_string());

    // Generate rust-toolchain.toml
    let toolchain_content = tera.render("rust-toolchain.toml", &context)?;
    fs::write(output_dir.join("rust-toolchain.toml"), toolchain_content)?;
    files_generated.push("rust-toolchain.toml".to_string());

    // Generate .gitignore
    let gitignore_content = tera.render(".gitignore", &context)?;
    fs::write(output_dir.join(".gitignore"), gitignore_content)?;
    files_generated.push(".gitignore".to_string());

    // Generate services
    for service in &plan.services {
        generate_service(output_dir, service, &plan, &mut tera, &mut context, &mut files_generated)?;
    }

    // Generate tools
    if let Some(tools) = &plan.tools {
        for tool in tools {
            generate_tool(output_dir, tool, &plan, &mut tera, &mut context, &mut files_generated)?;
        }
    }

    // Generate GitHub workflows
    write_github_workflows(output_dir, &plan, &mut tera, &mut context, &mut files_generated)?;

    Ok(files_generated)
}

fn generate_service(
    output_dir: &Path,
    service: &crate::types::ServiceConfig,
    _plan: &Plan,
    tera: &mut Tera,
    context: &mut Context,
    files: &mut Vec<String>,
) -> Result<()> {
    let service_dir = output_dir.join("services").join(&service.name);
    fs::create_dir_all(&service_dir)?;

    context.insert("service", service);

    let (template_prefix, main_file) = match service.service_type {
        ServiceType::Api => ("services/api", "main.rs"),
        ServiceType::ApiEdge => ("services/api-edge", "lib.rs"),
        ServiceType::Worker => ("services/worker", "lib.rs"),
        ServiceType::Library => ("services/library", "lib.rs"),
    };

    // Generate Cargo.toml
    let cargo_template = format!("{}/Cargo.toml", template_prefix);
    let cargo_content = tera.render(&cargo_template, context)?;
    fs::write(service_dir.join("Cargo.toml"), cargo_content)?;
    files.push(format!("services/{}/Cargo.toml", service.name));

    // Generate source file
    let src_dir = service_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    let src_template = format!("{}/{}", template_prefix, main_file);
    let src_content = tera.render(&src_template, context)?;
    fs::write(src_dir.join(main_file), src_content)?;
    files.push(format!("services/{}/src/{}", service.name, main_file));

    // Generate wrangler.toml for edge services
    if service.service_type == ServiceType::ApiEdge {
        let wrangler_template = format!("{}/wrangler.toml", template_prefix);
        let wrangler_content = tera.render(&wrangler_template, context)?;
        fs::write(service_dir.join("wrangler.toml"), wrangler_content)?;
        files.push(format!("services/{}/wrangler.toml", service.name));
    }

    Ok(())
}

fn generate_tool(
    output_dir: &Path,
    tool: &crate::types::ToolConfig,
    _plan: &Plan,
    tera: &mut Tera,
    context: &mut Context,
    files: &mut Vec<String>,
) -> Result<()> {
    let tool_dir = output_dir.join("tools").join(&tool.name);
    fs::create_dir_all(&tool_dir)?;

    context.insert("tool", tool);

    // Generate Cargo.toml
    let cargo_content = tera.render("tools/cli/Cargo.toml", context)?;
    fs::write(tool_dir.join("Cargo.toml"), cargo_content)?;
    files.push(format!("tools/{}/Cargo.toml", tool.name));

    // Generate main.rs
    let src_dir = tool_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    let main_content = tera.render("tools/cli/main.rs", context)?;
    fs::write(src_dir.join("main.rs"), main_content)?;
    files.push(format!("tools/{}/src/main.rs", tool.name));

    Ok(())
}

fn write_github_workflows(
    output_dir: &Path,
    _plan: &Plan,
    tera: &mut Tera,
    context: &mut Context,
    files: &mut Vec<String>,
) -> Result<()> {
    let workflow_dir = output_dir.join(".github/workflows");
    fs::create_dir_all(&workflow_dir)?;

    let ci_content = tera.render(".github/workflows/ci.yml", context)?;
    fs::write(workflow_dir.join("ci.yml"), ci_content)?;
    files.push(".github/workflows/ci.yml".to_string());

    Ok(())
}

fn load_templates(tera: &mut Tera) -> Result<()> {
    // Load templates from the templates directory
    let template_dir = std::env::current_exe()?
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find executable directory"))?
        .join("../../../templates");

    // If running in development, use the project templates directory
    let template_dir = if template_dir.exists() {
        template_dir
    } else {
        std::path::PathBuf::from("templates")
    };

    tera.add_raw_templates(vec![
        ("workspace.toml", include_str!("../templates/workspace.toml")),
        ("rust-toolchain.toml", include_str!("../templates/rust-toolchain.toml")),
        (".gitignore", include_str!("../templates/.gitignore")),
        ("services/api/Cargo.toml", include_str!("../templates/services/api/Cargo.toml")),
        ("services/api/main.rs", include_str!("../templates/services/api/main.rs")),
        ("services/api-edge/Cargo.toml", include_str!("../templates/services/api-edge/Cargo.toml")),
        ("services/api-edge/lib.rs", include_str!("../templates/services/api-edge/lib.rs")),
        ("services/api-edge/wrangler.toml", include_str!("../templates/services/api-edge/wrangler.toml")),
        ("services/worker/Cargo.toml", include_str!("../templates/services/worker/Cargo.toml")),
        ("services/worker/lib.rs", include_str!("../templates/services/worker/lib.rs")),
        ("services/library/Cargo.toml", include_str!("../templates/services/library/Cargo.toml")),
        ("services/library/lib.rs", include_str!("../templates/services/library/lib.rs")),
        ("tools/cli/Cargo.toml", include_str!("../templates/tools/cli/Cargo.toml")),
        ("tools/cli/main.rs", include_str!("../templates/tools/cli/main.rs")),
        (".github/workflows/ci.yml", include_str!("../templates/.github/workflows/ci.yml")),
    ])?;

    Ok(())
}