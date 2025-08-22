use anyhow::Result;
use clap::Parser;
use tracing::info;

mod cli;
mod git;
mod language_pack;
mod manifest;
mod render;
mod verify;

use cli::{Cli, Commands};
use manifest::{generate_manifest, write_manifest};
use render::{render_templates, RenderContext};
use verify::{verify_plan, verify_policy};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Apply {
            plan,
            seed,
            repo,
            policy,
            out,
            verify,
        } => {
            if verify {
                // Just verify, don't generate
                let _ = verify_plan(&plan)?;
                let _ = verify_policy(policy.as_deref())?;
                info!("Verification successful");
                return Ok(());
            }

            // Verify inputs
            let stack_plan = verify_plan(&plan)?;
            let policy_data = verify_policy(policy.as_deref())?;

            // Use seed or generate random
            let seed = seed.unwrap_or_else(|| {
                use rand::Rng;
                rand::thread_rng().gen()
            });

            info!("Generating scaffold with seed: {}", seed);

            // Create render context
            let ctx = RenderContext {
                plan: stack_plan,
                policy: policy_data,
                seed,
                out_dir: out.clone(),
            };

            // Render templates
            render_templates(&ctx)?;

            // Generate manifest
            let plan_content = std::fs::read_to_string(&plan)?;
            let rust_version = ctx
                .plan
                .toolchain
                .rust
                .as_ref()
                .map(|r| r.version.clone())
                .unwrap_or_else(|| "1.82".to_string());

            let manifest = generate_manifest(
                &plan_content,
                seed,
                &rust_version,
                "1.0.0", // template version
            )?;
            write_manifest(&manifest, &out)?;

            info!("Scaffold generated at: {:?}", out);

            // Handle repository push if specified
            if let Some(repo_spec) = repo {
                let git_ops = git::GitOps::new(&repo_spec)?;
                git_ops.push_to_repo(&out, "main")?;
            }

            Ok(())
        }
        Commands::Verify { plan, policy } => {
            let _ = verify_plan(&plan)?;
            let _ = verify_policy(policy.as_deref())?;
            info!("Verification successful");
            Ok(())
        }
    }
}
