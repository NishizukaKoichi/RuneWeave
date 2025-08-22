use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "runeweave")]
#[command(version)]
#[command(about = "Rust-Edge monorepo scaffolding tool", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Apply a plan to generate scaffold
    Apply {
        /// Path to plan.json
        #[arg(short, long, value_name = "FILE")]
        plan: PathBuf,

        /// Random seed for deterministic generation
        #[arg(long)]
        seed: Option<u64>,

        /// Repository to push to (e.g., github:owner/repo)
        #[arg(long)]
        repo: Option<String>,

        /// Path to policy file
        #[arg(long)]
        policy: Option<PathBuf>,

        /// Output directory
        #[arg(long, default_value = "./scaffold")]
        out: PathBuf,

        /// Verify only, don't generate
        #[arg(long)]
        verify: bool,
    },

    /// Verify a plan without generating
    Verify {
        /// Path to plan.json
        #[arg(short, long, value_name = "FILE")]
        plan: PathBuf,

        /// Path to policy file
        #[arg(long)]
        policy: Option<PathBuf>,
    },
}
