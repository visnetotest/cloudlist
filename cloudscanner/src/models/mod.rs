// cloudscanner/src/models/mod.rs

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Asset {
    pub id: String,
    pub asset_type: String,
    pub provider: String,
}
