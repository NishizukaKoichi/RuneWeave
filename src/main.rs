mod git;
mod manifest;
mod render;
mod types;
mod verify;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process;
use types::CliArgs;

#[derive(Parser)]
#[command(
    name = "runeweave",
    about = "Generate Rust-Edge monorepo scaffolds from Runeforge blueprints",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    #[command(about = "Apply a plan to generate scaffold")]
    Apply {
        #[arg(short = 'p', long, help = "Path to plan.json file")]
        plan: PathBuf,

        #[arg(long, help = "Seed for deterministic generation")]
        seed: Option<u64>,

        #[arg(long, help = "GitHub repository (e.g., github:owner/repo)")]
        repo: Option<String>,

        #[arg(long, help = "Policy file path")]
        policy: Option<PathBuf>,

        #[arg(long, help = "Output directory for local generation")]
        out: Option<PathBuf>,

        #[arg(long, help = "Verify only, don't generate")]
        verify: bool,

        #[arg(long, help = "Make repository public")]
        public: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Apply {
            plan,
            seed,
            repo,
            policy,
            out,
            verify,
            public,
        } => {
            let args = CliArgs {
                plan_path: plan,
                seed,
                repo,
                policy_path: policy,
                output_dir: out,
                verify_only: verify,
                public,
            };

            match run_apply(args).await {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    match e.downcast_ref::<types::RuneWeaveError>() {
                        Some(types::RuneWeaveError::SchemaValidation(_)) => 1,
                        Some(types::RuneWeaveError::PolicyViolation(_)) => 2,
                        Some(types::RuneWeaveError::GitError(_)) => 3,
                        _ => 4,
                    }
                }
            }
        }
    };

    process::exit(exit_code);
}

async fn run_apply(args: CliArgs) -> Result<()> {
    let plan_content = std::fs::read_to_string(&args.plan_path)?;
    let plan: types::Plan = serde_json::from_str(&plan_content)?;

    let policy = if let Some(policy_path) = &args.policy_path {
        let policy_content = std::fs::read_to_string(policy_path)?;
        Some(serde_yaml::from_str(&policy_content)?)
    } else {
        None
    };

    verify::validate_plan(&plan)?;
    
    if let Some(policy) = &policy {
        verify::check_policy(&plan, policy)?;
    }

    if args.verify_only {
        println!("✓ Validation passed");
        return Ok(());
    }

    let seed = args.seed.unwrap_or_else(|| {
        use rand::Rng;
        rand::thread_rng().gen()
    });

    println!("🔧 Generating scaffold with seed: {}", seed);

    let output_dir = args.output_dir.unwrap_or_else(|| {
        PathBuf::from(format!("./{}", plan.project.name))
    });

    let files = render::generate_scaffold(&plan, seed, &output_dir)?;
    
    let manifest = manifest::create_manifest(&plan, &files, seed)?;
    let manifest_path = output_dir.join("weave.manifest.json");
    std::fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)?,
    )?;

    println!("✓ Generated {} files", files.len());

    verify::verify_build(&output_dir)?;
    
    if let Some(repo) = &args.repo {
        git::push_to_repo(&output_dir, repo, args.public).await?;
        println!("✓ Pushed to {}", repo);
    }

    println!("✨ Successfully generated scaffold at: {}", output_dir.display());
    Ok(())
}