// cloudscanner/src/reporter/mod.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tracing::info;

use crate::models::provider::Resource;

#[cfg(test)]
pub mod tests;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanReport {
    pub timestamp: String,
    pub total_resources: usize,
    pub resources_by_type: HashMap<String, usize>,
    pub resources: Vec<ResourceReport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceReport {
    pub asset_type: String,
    pub id: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Console,
    Json,
    Yaml,
}

pub struct Reporter {
    output_format: OutputFormat,
}

impl Reporter {
    pub fn new() -> Self {
        Self {
            output_format: OutputFormat::Console,
        }
    }

    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    pub fn report(&self, resources: &[Resource]) -> Result<()> {
        info!("Generating report with {} resources", resources.len());

        let report = self.generate_report(resources)?;

        match self.output_format {
            OutputFormat::Console => self.print_console_report(&report),
            OutputFormat::Json => self.print_json_report(&report)?,
            OutputFormat::Yaml => self.print_yaml_report(&report)?,
        }

        Ok(())
    }

    fn generate_report(&self, resources: &[Resource]) -> Result<ScanReport> {
        let mut resources_by_type = HashMap::new();
        let mut resource_reports = Vec::new();

        for resource in resources {
            *resources_by_type.entry(resource.asset_type.clone()).or_insert(0) += 1;
            
            resource_reports.push(ResourceReport {
                asset_type: resource.asset_type.clone(),
                id: resource.id.clone(),
                metadata: resource.metadata.clone(),
            });
        }

        Ok(ScanReport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_resources: resources.len(),
            resources_by_type,
            resources: resource_reports,
        })
    }

    fn print_console_report(&self, report: &ScanReport) {
        println!("\n=== Cloud Scanner Report ===");
        println!("Timestamp: {}", report.timestamp);
        println!("Total Resources: {}", report.total_resources);
        
        if !report.resources_by_type.is_empty() {
            println!("\nResources by Type:");
            for (asset_type, count) in &report.resources_by_type {
                println!("  {}: {}", asset_type, count);
            }
        }

        if !report.resources.is_empty() {
            println!("\nDiscovered Resources:");
            for resource in &report.resources {
                println!("  [{}] {}", resource.asset_type, resource.id);
                if !resource.metadata.is_empty() {
                    for (key, value) in &resource.metadata {
                        println!("    {}: {}", key, value);
                    }
                }
            }
        }
        println!("============================\n");
    }

    fn print_json_report(&self, report: &ScanReport) -> Result<()> {
        let json = serde_json::to_string_pretty(report)
            .map_err(|e| anyhow::anyhow!("Failed to serialize report to JSON: {}", e))?;
        println!("{}", json);
        Ok(())
    }

    fn print_yaml_report(&self, report: &ScanReport) -> Result<()> {
        let yaml = serde_yaml::to_string(report)
            .map_err(|e| anyhow::anyhow!("Failed to serialize report to YAML: {}", e))?;
        println!("{}", yaml);
        Ok(())
    }
}
