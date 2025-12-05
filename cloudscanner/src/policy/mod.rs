// cloudscanner/src/policy/mod.rs

pub mod policy;

use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::models::provider::Resource;
use policy::Policy;

pub struct PolicyEngine {
    policies: Vec<Policy>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            policies: vec![],
        }
    }

    pub fn load_policies(&mut self, path: &str) -> Result<()> {
        println!("Loading policies from: {}", path);
        let policy_path = Path::new(path);

        for entry in fs::read_dir(policy_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && (path.extension().map_or(false, |ext| ext == "yaml" || ext == "yml")) {
                let policy_content = fs::read_to_string(&path)?;
                let policy: Policy = serde_yaml::from_str(&policy_content)?;
                println!("Loaded policy: {}", policy.name);
                self.policies.push(policy);
            }
        }
        Ok(())
    }

    pub fn evaluate(&self, resources: &[Resource]) {
        println!(
            "Policy engine evaluating {} resources against {} policies...",
            resources.len(),
            self.policies.len()
        );

        for resource in resources {
            for policy in &self.policies {
                if resource.asset_type == policy.asset_type {
                    let mut is_match = true;
                    for rule in &policy.rules {
                        if let Some(metadata_value) = resource.metadata.get(&rule.key) {
                            if metadata_value != &rule.value {
                                is_match = false;
                                break;
                            }
                        } else {
                            is_match = false;
                            break;
                        }
                    }

                    if is_match {
                        println!(
                            "Violation detected: Resource {} matches policy {}",
                            resource.id, policy.name
                        );
                    }
                }
            }
        }
    }
}
