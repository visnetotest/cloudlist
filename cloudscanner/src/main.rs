mod config;
mod engine;
mod models;
mod policy;
mod reporter;

use clap::Parser;

/// A simple CLI for scanning cloud assets
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[clap(short, long)]
    config: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Cloud Scanner PoC2: Initializing...");

    let args = Args::parse();

    // 1. Load Configuration
    let _config = config::load_config(&args.config)?;
    println!("Configuration loaded from: {}", &args.config);

    // 2. Initialize Core Components
    let discovery_engine = engine::DiscoveryEngine::new();
    let policy_engine = policy::PolicyEngine::new();
    let reporter = reporter::Reporter::new();

    // 3. Run the Discovery -> Evaluate -> Report Pipeline
    discovery_engine.run().await;
    policy_engine.evaluate();
    reporter.report();

    println!("Cloud Scanner run complete.");
    Ok(())
}
