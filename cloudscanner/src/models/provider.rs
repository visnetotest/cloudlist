// cloudscanner/src/models/provider.rs

use anyhow::Result;

// Represents a discovered cloud asset.
// This will be expanded as we add more structured data.
#[derive(Debug)]
pub struct Resource {
    pub asset_type: String,
    pub id: String,
    pub metadata: std::collections::HashMap<String, String>,
}

/// A C-compatible representation of a Rust trait object.
/// A trait object is a "fat pointer" consisting of a data pointer and a vtable pointer.
#[repr(C)]
pub struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}


/// The trait that all provider plugins must implement.
pub trait Provider: Send + Sync {
    /// Returns the name of the provider (e.g., "aws", "gcp").
    fn name(&self) -> &str;

    /// Runs the discovery process and returns a list of resources.
    /// This is a blocking, synchronous operation. The caller is responsible
    /// for running it in a separate thread if needed to avoid blocking.
    fn discover(&self) -> Result<Vec<Resource>>;
}
