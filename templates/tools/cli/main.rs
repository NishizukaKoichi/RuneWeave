//! {{ tool.description }}

use clap::Parser;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "{{ tool.name }}", about = "{{ tool.description }}", version)]
struct Cli {
    #[arg(short, long, help = "Enable verbose output")]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    tracing::info!("Running {{ tool.name }}");
    
    Ok(())
}