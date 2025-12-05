// cloudscanner/src/engine/mod.rs

use anyhow::Result;
use libloading::{Library, Symbol};
use std::fs;
use std::path::Path;
use std::mem;

use crate::models::provider::{Provider, TraitObject, Resource};

type ProviderCreator = unsafe extern "C" fn() -> TraitObject;

pub struct DiscoveryEngine {
    providers: Vec<Box<dyn Provider>>,
    // We need to keep the libraries loaded for the duration of the program
    loaded_libs: Vec<Library>,
}

impl DiscoveryEngine {
    pub fn new() -> Self {
        Self {
            providers: vec![],
            loaded_libs: vec![],
        }
    }

    pub unsafe fn load_providers(&mut self, path: &str) -> Result<()> {
        println!("Loading providers from: {}", path);
        let plugin_path = Path::new(path);

        for entry in fs::read_dir(plugin_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "so" || ext == "dylib" || ext == "dll") {
                println!("Found potential plugin: {:?}", &path);
                
                let lib = Library::new(&path)?;
                self.loaded_libs.push(lib);

                let lib = self.loaded_libs.last().unwrap();
                let constructor: Symbol<ProviderCreator> = lib.get(b"_create_provider")?;
                let trait_object = constructor();

                // Transmute the C-compatible TraitObject back to a Rust Box<dyn Provider>
                let provider: Box<dyn Provider> = mem::transmute(trait_object);
                self.providers.push(provider);
            }
        }

        Ok(())
    }

    pub async fn run(&self) -> Vec<Resource> {
        println!("Discovery engine running...");
        let mut all_resources = Vec::new();

        for provider in &self.providers {
            println!("Running discovery for provider: {}", provider.name());
            // This is blocking, but for the PoC, let's keep it simple.
            // A real implementation would use `spawn_blocking`.
            match provider.discover() {
                Ok(mut resources) => {
                    println!("Discovered {} resources", resources.len());
                    all_resources.append(&mut resources);
                }
                Err(e) => {
                    eprintln!("Error discovering resources for provider {}: {}", provider.name(), e);
                }
            }
        }
        all_resources
    }
}
