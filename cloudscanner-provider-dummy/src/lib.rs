// cloudscanner-provider-dummy/src/lib.rs

use cloudscanner::models::provider::{Provider, Resource, TraitObject};
use anyhow::Result;
use std::collections::HashMap;
use std::mem;


struct DummyProvider;

impl Provider for DummyProvider {
    fn name(&self) -> &str {
        "dummy"
    }

    fn discover(&self) -> Result<Vec<Resource>> {
        println!("Dummy provider is discovering resources...");
        
        let mut metadata = HashMap::new();
        metadata.insert("public".to_string(), "true".to_string());

        let resource = Resource {
            asset_type: "s3-bucket".to_string(),
            id: "dummy-s3-bucket-123".to_string(),
            metadata,
        };

        Ok(vec![resource])
    }
}

/// The function that creates an instance of the provider.
#[no_mangle]
pub extern "C" fn _create_provider() -> TraitObject {
    // Create a concrete instance of the provider
    let provider = DummyProvider;

    // Box it, so it has a stable address
    let boxed_provider: Box<dyn Provider> = Box::new(provider);

    // Transmute the fat pointer (Box<dyn Provider>) to our C-compatible TraitObject.
    // This is unsafe, but it's the core of the plugin interface.
    unsafe {
        mem::transmute(boxed_provider)
    }
}
