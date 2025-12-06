// cloudscanner/src/main.rs

use clap::Parser;
use cloudscanner::config;
use cloudscanner::engine;
use cloudscanner::policy;
use cloudscanner::reporter;
use tracing::{info, error};
use tracing_subscriber;

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

    /// Output format (console, json, yaml)
    #[clap(long, default_value = "console")]
    output_format: String,

    /// Log level (trace, debug, info, warn, error)
    #[clap(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
        .with_target(false)
        .init();

    info!("Cloud Scanner PoC2: Initializing...");

    // 1. Load Configuration
    let _config = match config::load_config(&args.config) {
        Ok(cfg) => {
            info!("Configuration loaded from: {}", &args.config);
            cfg
        }
        Err(e) => {
            error!("Failed to load configuration from {}: {}", &args.config, e);
            return Err(e.into());
        }
    };

    // 2. Initialize Core Components
    let mut discovery_engine = engine::DiscoveryEngine::new();
    let mut policy_engine = policy::PolicyEngine::new();
    
    let output_format = match args.output_format.as_str() {
        "json" => reporter::OutputFormat::Json,
        "yaml" => reporter::OutputFormat::Yaml,
        _ => reporter::OutputFormat::Console,
    };
    
    let reporter = reporter::Reporter::new().with_format(output_format);

    // 3. Load Providers and Policies
    if let Err(e) = discovery_engine.load_providers_from_config(&_config).await {
        error!("Failed to load providers from configuration");
        return Err(e.into());
    }

    if let Err(e) = policy_engine.load_policies(&args.policies) {
        error!("Failed to load policies from {}: {}", &args.policies, e);
        return Err(e);
    }

    // 4. Run the Discovery -> Evaluate -> Report Pipeline
    info!("Starting discovery process...");
    let discovered_resources = discovery_engine.run().await;
    info!("Discovered {} total resources", discovered_resources.len());

    info!("Starting policy evaluation...");
    policy_engine.evaluate(&discovered_resources);

    info!("Generating report...");
    if let Err(e) = reporter.report(&discovered_resources) {
        error!("Failed to generate report: {}", e);
        return Err(e.into());
    }

    info!("Cloud Scanner run complete.");
    Ok(())
}
