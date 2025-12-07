// cloudscanner/src/models/provider.rs

use anyhow::Result;
use async_trait::async_trait;

// Represents a discovered cloud asset.
// This will be expanded as we add more structured data.
#[derive(Debug, Clone)]
pub struct Resource {
    pub asset_type: String,
    pub id: String,
    pub metadata: std::collections::HashMap<String, String>,
}

impl Resource {
    pub fn new(asset_type: String, id: String) -> Self {
        Self {
            asset_type,
            id,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Returns a unique identifier for this resource across providers
    pub fn unique_id(&self) -> String {
        format!("{}:{}", self.asset_type, self.id)
    }
}

/// Provider metadata and capabilities
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub supported_resource_types: Vec<String>,
}

/// The async trait that all provider plugins must implement.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Returns information about the provider
    fn info(&self) -> ProviderInfo;

    /// Returns the name of the provider (e.g., "aws", "gcp").
    /// Default implementation returns the name from info(), but providers can override for efficiency.
    fn name(&self) -> String {
        self.info().name.clone()
    }

    /// Runs the discovery process asynchronously and returns a list of resources.
    async fn discover(&self) -> Result<Vec<Resource>>;

    /// Optional: Health check for the provider
    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    /// Optional: Cleanup resources used by the provider
    async fn cleanup(&self) -> Result<()> {
        Ok(())
    }
}

/// Legacy sync trait for backward compatibility during migration
pub trait SyncProvider: Send + Sync {
    /// Returns the name of the provider (e.g., "aws", "gcp").
    fn name(&self) -> &str;

    /// Runs the discovery process and returns a list of resources.
    fn discover(&self) -> Result<Vec<Resource>>;
}

/// Adapter to convert sync providers to async providers
pub struct SyncToAsyncAdapter<T: SyncProvider> {
    inner: T,
}

impl<T: SyncProvider> SyncToAsyncAdapter<T> {
    pub fn new(provider: T) -> Self {
        Self { inner: provider }
    }
}

#[async_trait]
impl<T: SyncProvider> Provider for SyncToAsyncAdapter<T>
where
    T: 'static, // Required for spawn_blocking
{
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: self.inner.name().to_string(),
            version: "1.0.0".to_string(),
            description: "Legacy sync provider".to_string(),
            supported_resource_types: vec![],
        }
    }

    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    async fn discover(&self) -> Result<Vec<Resource>> {
        // Use spawn_blocking to run sync operations in async context
        let provider_name = self.inner.name().to_string();
        tokio::task::spawn_blocking(move || {
            // Create a new instance since we can't move &self
            // This is a limitation during migration - in practice, sync providers
            // should be refactored to be async
            Err(anyhow::anyhow!("Sync provider '{}' needs to be migrated to async", provider_name))
        }).await.map_err(|e| anyhow::anyhow!("Task join error for provider: {}", e))?
    }
}

/// A C-compatible representation of a Rust trait object.
/// A trait object is a "fat pointer" consisting of a data pointer and a vtable pointer.
#[repr(C)]
pub struct TraitObject {
    pub data: *mut (),
    pub vtable: *mut (),
}

/// Plugin interface for dynamic loading
pub trait Plugin: Send + Sync {
    /// Creates a new provider instance
    fn create_provider(&self) -> Box<dyn Provider>;
    
    /// Returns plugin metadata
    fn plugin_info(&self) -> PluginInfo;
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub provider_name: String,
}

// Add C-compatible imports
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// Safe C-compatible plugin interface using function pointers
#[repr(C)]
pub struct CPluginInterface {
    /// Get plugin name
    pub get_name: unsafe extern "C" fn() -> *mut c_char,
    
    /// Get plugin version
    pub get_version: unsafe extern "C" fn() -> *mut c_char,
    
    /// Create provider instance - returns an opaque pointer
    pub create_provider: unsafe extern "C" fn() -> *mut std::ffi::c_void,
    
    /// Get provider name from instance
    pub get_provider_name: unsafe extern "C" fn(provider: *mut std::ffi::c_void) -> *mut c_char,
    
    /// Discover resources - returns C-compatible result
    pub discover: unsafe extern "C" fn(provider: *mut std::ffi::c_void) -> CPluginResult,
    
    /// Cleanup provider instance
    pub cleanup_provider: unsafe extern "C" fn(provider: *mut std::ffi::c_void),
    
    /// Cleanup plugin result
    pub cleanup_result: unsafe extern "C" fn(result: CPluginResult),
}

/// C-compatible resource structure for plugin interface
#[repr(C)]
pub struct CResource {
    pub asset_type: *mut c_char,
    pub id: *mut c_char,
    pub metadata: *mut CMetadata,
    pub metadata_count: c_int,
}

/// C-compatible metadata key-value pair
#[repr(C)]
pub struct CMetadata {
    pub key: *mut c_char,
    pub value: *mut c_char,
}

/// C-compatible resource array
#[repr(C)]
pub struct CResourceArray {
    pub resources: *mut CResource,
    pub count: c_int,
}

/// Plugin result with error handling
#[repr(C)]
pub struct CPluginResult {
    pub resources: CResourceArray,
    pub error_message: *mut c_char,
    pub success: bool,
}

/// Helper functions for C string conversion with validation
pub fn string_to_c_char(s: &str) -> Result<*mut c_char, &'static str> {
    // Validate string length to prevent excessive memory allocation
    const MAX_STRING_LENGTH: usize = 1024;
    if s.len() > MAX_STRING_LENGTH {
        return Err("String exceeds maximum allowed length");
    }
    
    // Validate for null bytes which could cause issues in C strings
    if s.contains('\0') {
        return Err("String contains null bytes");
    }
    
    match CString::new(s) {
        Ok(c_string) => Ok(c_string.into_raw()),
        Err(_) => Err("Failed to create C string from input"),
    }
}

pub unsafe fn c_char_to_string(s: *mut c_char) -> Result<String, &'static str> {
    if s.is_null() {
        return Ok(String::new());
    }
    
    // Validate string length to prevent excessive memory allocation
    let c_str = CStr::from_ptr(s);
    let str_bytes = c_str.to_bytes();
    
    const MAX_STRING_LENGTH: usize = 1024;
    if str_bytes.len() > MAX_STRING_LENGTH {
        return Err("C string exceeds maximum allowed length");
    }
    
    // Validate for valid UTF-8 sequences
    match std::str::from_utf8(str_bytes) {
        Ok(valid_str) => Ok(valid_str.to_string()),
        Err(_) => Ok(c_str.to_string_lossy().into_owned()), // Fallback to lossy conversion
    }
}

pub unsafe fn free_c_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// Convert Rust resources to C-compatible format with safety checks
pub fn resources_to_c(resources: &[Resource]) -> Result<CResourceArray, &'static str> {
    const MAX_RESOURCES: usize = 10000;
    const MAX_METADATA_PER_RESOURCE: usize = 100;
    
    let count = resources.len();
    
    // Validate resource count to prevent excessive memory allocation
    if count > MAX_RESOURCES {
        return Err("Number of resources exceeds maximum allowed limit");
    }
    
    let count_c = count as c_int;
    if count_c == 0 {
        return Ok(CResourceArray {
            resources: std::ptr::null_mut(),
            count: 0,
        });
    }

    // Safe allocation with proper error handling
    let layout = std::alloc::Layout::array::<CResource>(count)
        .map_err(|_| "Failed to create layout for resource array")?;
    
    let c_resources = unsafe {
        let ptr = std::alloc::alloc(layout) as *mut CResource;
        if ptr.is_null() {
            return Err("Failed to allocate memory for resource array");
        }
        
        // Initialize all bytes to zero for safety
        std::ptr::write_bytes(ptr, 0, layout.size());
        
        for (i, resource) in resources.iter().enumerate() {
            // Validate metadata count
            if resource.metadata.len() > MAX_METADATA_PER_RESOURCE {
                // Cleanup already allocated memory before returning error
                std::alloc::dealloc(ptr as *mut u8, layout);
                return Err("Resource metadata exceeds maximum allowed limit");
            }
            
            let c_metadata = if resource.metadata.is_empty() {
                std::ptr::null_mut()
            } else {
                let metadata_count = resource.metadata.len();
                let metadata_layout = std::alloc::Layout::array::<CMetadata>(metadata_count)
                    .map_err(|_| "Failed to create layout for metadata array")?;
                let metadata_ptr = std::alloc::alloc(metadata_layout) as *mut CMetadata;
                
                if metadata_ptr.is_null() {
                    // Cleanup already allocated memory before returning error
                    std::alloc::dealloc(ptr as *mut u8, layout);
                    return Err("Failed to allocate memory for metadata array");
                }
                
                // Initialize metadata bytes to zero
                std::ptr::write_bytes(metadata_ptr, 0, metadata_layout.size());
                
                let mut metadata_success = true;
                for (j, (key, value)) in resource.metadata.iter().enumerate() {
                    // Safe pointer arithmetic with bounds checking
                    if j >= metadata_count {
                        metadata_success = false;
                        break;
                    }
                    
                    let metadata_item = &mut *metadata_ptr.add(j);
                    match string_to_c_char(key) {
                        Ok(key_ptr) => metadata_item.key = key_ptr,
                        Err(_) => {
                            metadata_success = false;
                            break;
                        }
                    }
                    
                    match string_to_c_char(value) {
                        Ok(value_ptr) => metadata_item.value = value_ptr,
                        Err(_) => {
                            // Cleanup key string on failure
                            free_c_string(metadata_item.key);
                            metadata_success = false;
                            break;
                        }
                    }
                }
                
                if !metadata_success {
                    // Cleanup all allocated metadata on failure
                    for k in 0..metadata_count {
                        let metadata_item = &*metadata_ptr.add(k);
                        free_c_string(metadata_item.key);
                        free_c_string(metadata_item.value);
                    }
                    std::alloc::dealloc(metadata_ptr as *mut u8, metadata_layout);
                    std::alloc::dealloc(ptr as *mut u8, layout);
                    return Err("Failed to convert metadata to C format");
                }
                
                metadata_ptr
            };

            // Safe pointer access with bounds checking
            let resource_ptr = ptr.add(i);
            let asset_type_ptr = match string_to_c_char(&resource.asset_type) {
                Ok(ptr) => ptr,
                Err(e) => {
                    // Cleanup metadata on failure
                    if !c_metadata.is_null() {
                        free_c_metadata_array(c_metadata, resource.metadata.len());
                    }
                    // Cleanup previously allocated resources
                    for k in 0..i {
                        cleanup_resource(&*ptr.add(k));
                    }
                    std::alloc::dealloc(ptr as *mut u8, layout);
                    return Err(e);
                }
            };
            
            let id_ptr = match string_to_c_char(&resource.id) {
                Ok(ptr) => ptr,
                Err(e) => {
                    // Cleanup asset type and metadata on failure
                    free_c_string(asset_type_ptr);
                    if !c_metadata.is_null() {
                        free_c_metadata_array(c_metadata, resource.metadata.len());
                    }
                    // Cleanup previously allocated resources
                    for k in 0..i {
                        cleanup_resource(&*ptr.add(k));
                    }
                    std::alloc::dealloc(ptr as *mut u8, layout);
                    return Err(e);
                }
            };

            *resource_ptr = CResource {
                asset_type: asset_type_ptr,
                id: id_ptr,
                metadata: c_metadata,
                metadata_count: resource.metadata.len() as c_int,
            };
        }
        
        ptr
    };

    Ok(CResourceArray {
        resources: c_resources,
        count: count_c,
    })
}

/// Helper function to cleanup metadata array
unsafe fn free_c_metadata_array(metadata: *mut CMetadata, count: usize) {
    if metadata.is_null() {
        return;
    }
    
    for i in 0..count {
        let metadata_item = &*metadata.add(i);
        free_c_string(metadata_item.key);
        free_c_string(metadata_item.value);
    }
    
    let layout = std::alloc::Layout::array::<CMetadata>(count)
        .unwrap_or_else(|_| std::alloc::Layout::new::<CMetadata>());
    std::alloc::dealloc(metadata as *mut u8, layout);
}

/// Helper function to cleanup a single resource
unsafe fn cleanup_resource(resource: &CResource) {
    free_c_string(resource.asset_type);
    free_c_string(resource.id);
    
    if !resource.metadata.is_null() && resource.metadata_count > 0 {
        free_c_metadata_array(resource.metadata, resource.metadata_count as usize);
    }
}

/// Free C-compatible resource array with enhanced safety
pub fn free_c_resources(array: CResourceArray) {
    if array.resources.is_null() {
        return;
    }
    
    // Validate count to prevent underflow/overflow
    if array.count < 0 {
        return;
    }
    
    let count = array.count as usize;
    const MAX_RESOURCES: usize = 10000;
    
    if count > MAX_RESOURCES {
        return; // Safety limit exceeded
    }

    unsafe {
        // Validate layout before proceeding
        let layout = match std::alloc::Layout::array::<CResource>(count) {
            Ok(layout) => layout,
            Err(_) => return, // Invalid layout, cannot safely deallocate
        };
        
        for i in 0..count {
            // Safe pointer access with bounds checking
            let resource_ptr = array.resources.add(i);
            if resource_ptr.is_null() {
                break;
            }
            
            let resource = &*resource_ptr;
            
            // Cleanup strings
            free_c_string(resource.asset_type);
            free_c_string(resource.id);
            
            // Cleanup metadata with validation
            if !resource.metadata.is_null() && resource.metadata_count > 0 {
                let metadata_count = resource.metadata_count as usize;
                if metadata_count <= MAX_METADATA_PER_RESOURCE {
                    for j in 0..metadata_count {
                        let metadata_ptr = resource.metadata.add(j);
                        if metadata_ptr.is_null() {
                            break;
                        }
                        let metadata = &*metadata_ptr;
                        free_c_string(metadata.key);
                        free_c_string(metadata.value);
                    }
                    
                    // Deallocate metadata array
                    if let Ok(metadata_layout) = std::alloc::Layout::array::<CMetadata>(metadata_count) {
                        std::alloc::dealloc(resource.metadata as *mut u8, metadata_layout);
                    }
                }
            }
        }
        
        // Deallocate resource array
        std::alloc::dealloc(array.resources as *mut u8, layout);
    }
}

const MAX_METADATA_PER_RESOURCE: usize = 100;