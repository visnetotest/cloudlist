// cloudscanner/src/policy/policy.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Policy {
    pub name: String,
    pub asset_type: String,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub key: String,
    pub value: String,
}
