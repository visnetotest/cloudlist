// cloudscanner/src/main.rs

use clap::Parser;
use cloudscanner::config;
use cloudscanner::engine;
use cloudscanner::policy;
use cloudscanner::reporter;

/// A simple CLI for scanning cloud assets
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Path to the configuration file
    #[clap(short, long)]
    config: String,

    /// Directory containing provider plugins
    #[clap(short, long, default_value = "plugins")]
    plugins: String,

    /// Directory containing policy files
    #[clap(short = 'l', long, default_value = "policies")]
    policies: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Cloud Scanner PoC2: Initializing...");

    let args = Args::parse();

    // 1. Load Configuration
    let _config = config::load_config(&args.config)?;
    println!("Configuration loaded from: {}", &args.config);

    // 2. Initialize Core Components
    let mut discovery_engine = engine::DiscoveryEngine::new();
    let mut policy_engine = policy::PolicyEngine::new();
    let reporter = reporter::Reporter::new();

    // 3. Load Providers and Policies
    unsafe {
        discovery_engine.load_providers(&args.plugins)?;
    }
    policy_engine.load_policies(&args.policies)?;

    // 4. Run the Discovery -> Evaluate -> Report Pipeline
    let discovered_resources = discovery_engine.run().await;
    policy_engine.evaluate(&discovered_resources);
    reporter.report();

    println!("Cloud Scanner run complete.");
    Ok(())
}
