// cloudscanner/src/policy/mod.rs

pub mod policy;

use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{info, warn, error};

use crate::models::provider::Resource;
use policy::Policy;

#[cfg(test)]
pub mod tests;

#[derive(thiserror::Error, Debug)]
pub enum PolicyError {
    #[error("Failed to read policy file: {0}")]
    FileReadError(#[from] std::io::Error),
    
    #[error("Failed to parse policy YAML: {0}")]
    ParseError(#[from] serde_yaml::Error),
    
    #[error("Policy directory not found: {0}")]
    DirectoryNotFoundError(String),
}

pub struct PolicyViolation {
    pub resource_id: String,
    pub resource_type: String,
    pub policy_name: String,
    pub violated_rules: Vec<String>,
}

pub struct PolicyEngine {
    policies: Vec<Policy>,
    violations: Vec<PolicyViolation>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            policies: vec![],
            violations: vec![],
        }
    }

    pub fn load_policies(&mut self, path: &str) -> Result<()> {
        info!("Loading policies from: {}", path);
        let policy_path = Path::new(path);

        if !policy_path.exists() {
            warn!("Policy directory does not exist: {}", path);
            return Ok(());
        }

        let mut loaded_count = 0;
        for entry in fs::read_dir(policy_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && (path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml")) {
                match self.load_policy_file(&path) {
                    Ok(policy_name) => {
                        info!("Loaded policy: {}", policy_name);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        error!("Failed to load policy {:?}: {}", &path, e);
                        // Continue loading other policies
                    }
                }
            }
        }
        
        info!("Successfully loaded {} policies from {}", loaded_count, path);
        Ok(())
    }

    fn load_policy_file(&mut self, path: &Path) -> Result<String> {
        let policy_content = fs::read_to_string(path)?;
        let policy: Policy = serde_yaml::from_str(&policy_content)?;
        let policy_name = policy.name.clone();
        self.policies.push(policy);
        Ok(policy_name)
    }

    pub fn evaluate(&mut self, resources: &[Resource]) {
        info!(
            "Policy engine evaluating {} resources against {} policies...",
            resources.len(),
            self.policies.len()
        );

        self.violations.clear();
        let mut total_violations = 0;

        for resource in resources {
            for policy in &self.policies {
                if let Some(violation) = self.evaluate_resource_against_policy(resource, policy) {
                    total_violations += 1;
                    self.violations.push(violation);
                }
            }
        }

        if total_violations > 0 {
            warn!("Policy evaluation complete: {} violations detected", total_violations);
        } else {
            info!("Policy evaluation complete: no violations detected");
        }
    }

    fn evaluate_resource_against_policy(&self, resource: &Resource, policy: &Policy) -> Option<PolicyViolation> {
        if resource.asset_type != policy.asset_type {
            return None;
        }

        let mut violated_rules = Vec::new();
        
        for rule in &policy.rules {
            match resource.metadata.get(&rule.key) {
                Some(metadata_value) => {
                    if metadata_value != &rule.value {
                        violated_rules.push(format!("{}: expected '{}', found '{}'", rule.key, rule.value, metadata_value));
                    }
                }
                None => {
                    violated_rules.push(format!("{}: missing required metadata", rule.key));
                }
            }
        }

        if !violated_rules.is_empty() {
            Some(PolicyViolation {
                resource_id: resource.id.clone(),
                resource_type: resource.asset_type.clone(),
                policy_name: policy.name.clone(),
                violated_rules,
            })
        } else {
            None
        }
    }

    pub fn get_violations(&self) -> &[PolicyViolation] {
        &self.violations
    }

    pub fn get_violations_count(&self) -> usize {
        self.violations.len()
    }
}
