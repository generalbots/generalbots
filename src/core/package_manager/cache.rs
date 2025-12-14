//! Download Cache Module
//!
//! Provides caching functionality for third-party downloads.
//! Files are cached in `botserver-installers/` directory and reused
//! on subsequent runs, allowing offline installation.
//!
//! Configuration is read from `3rdparty.toml` at the botserver root.

use anyhow::{Context, Result};
use log::{debug, info, trace, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Default cache directory relative to botserver root
const DEFAULT_CACHE_DIR: &str = "botserver-installers";

/// Configuration file name
const CONFIG_FILE: &str = "3rdparty.toml";

/// Third-party dependencies configuration
#[derive(Debug, Deserialize, Default)]
pub struct ThirdPartyConfig {
    #[serde(default)]
    pub cache_settings: CacheSettings,
    #[serde(default)]
    pub components: HashMap<String, ComponentDownload>,
    #[serde(default)]
    pub models: HashMap<String, ComponentDownload>,
}

/// Cache settings
#[derive(Debug, Deserialize)]
pub struct CacheSettings {
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            cache_dir: default_cache_dir(),
        }
    }
}

fn default_cache_dir() -> String {
    DEFAULT_CACHE_DIR.to_string()
}

/// Component download configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ComponentDownload {
    pub name: String,
    pub url: String,
    pub filename: String,
    #[serde(default)]
    pub sha256: String,
}

/// Download cache manager
#[derive(Debug)]
pub struct DownloadCache {
    /// Base path for the botserver (where 3rdparty.toml lives)
    base_path: PathBuf,
    /// Cache directory path
    cache_dir: PathBuf,
    /// Loaded configuration
    config: ThirdPartyConfig,
}

impl DownloadCache {
    /// Create a new download cache manager
    ///
    /// # Arguments
    /// * `base_path` - Base path for botserver (typically current directory or botserver root)
    ///
    /// # Environment Variables
    /// * `BOTSERVER_INSTALLERS_PATH` - Override path to pre-downloaded installers directory
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        let config = Self::load_config(&base_path)?;

        // Check for BOTSERVER_INSTALLERS_PATH env var first (for testing/offline installs)
        let cache_dir = if let Ok(installers_path) = std::env::var("BOTSERVER_INSTALLERS_PATH") {
            let path = PathBuf::from(&installers_path);
            if path.exists() {
                info!("Using installers from BOTSERVER_INSTALLERS_PATH: {:?}", path);
                path
            } else {
                warn!("BOTSERVER_INSTALLERS_PATH set but path doesn't exist: {:?}", path);
                base_path.join(&config.cache_settings.cache_dir)
            }
        } else {
            base_path.join(&config.cache_settings.cache_dir)
        };

        // Ensure cache directory exists
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .with_context(|| format!("Failed to create cache directory: {:?}", cache_dir))?;
            info!("Created cache directory: {:?}", cache_dir);
        }

        Ok(Self {
            base_path,
            cache_dir,
            config,
        })
    }

    /// Load configuration from 3rdparty.toml
    fn load_config(base_path: &Path) -> Result<ThirdPartyConfig> {
        let config_path = base_path.join(CONFIG_FILE);

        if !config_path.exists() {
            debug!(
                "No {} found at {:?}, using defaults",
                CONFIG_FILE, config_path
            );
            return Ok(ThirdPartyConfig::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let config: ThirdPartyConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

        debug!(
            "Loaded {} with {} components and {} models",
            CONFIG_FILE,
            config.components.len(),
            config.models.len()
        );

        Ok(config)
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get the base path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Check if a file is cached
    ///
    /// # Arguments
    /// * `filename` - The filename to check in the cache
    pub fn is_cached(&self, filename: &str) -> bool {
        let cached_path = self.cache_dir.join(filename);
        if cached_path.exists() {
            // Also check that file is not empty
            if let Ok(metadata) = fs::metadata(&cached_path) {
                return metadata.len() > 0;
            }
        }
        false
    }

    /// Get the cached file path if it exists
    ///
    /// # Arguments
    /// * `filename` - The filename to get from cache
    pub fn get_cached_path(&self, filename: &str) -> Option<PathBuf> {
        let cached_path = self.cache_dir.join(filename);
        if self.is_cached(filename) {
            Some(cached_path)
        } else {
            None
        }
    }

    /// Get the path where a file should be cached
    ///
    /// # Arguments
    /// * `filename` - The filename
    pub fn get_cache_path(&self, filename: &str) -> PathBuf {
        self.cache_dir.join(filename)
    }

    /// Look up component download info by component name
    ///
    /// # Arguments
    /// * `component` - Component name (e.g., "drive", "tables", "llm")
    pub fn get_component(&self, component: &str) -> Option<&ComponentDownload> {
        self.config.components.get(component)
    }

    /// Look up model download info by model name
    ///
    /// # Arguments
    /// * `model` - Model name (e.g., "deepseek_small", "bge_embedding")
    pub fn get_model(&self, model: &str) -> Option<&ComponentDownload> {
        self.config.models.get(model)
    }

    /// Get all component downloads
    pub fn all_components(&self) -> &HashMap<String, ComponentDownload> {
        &self.config.components
    }

    /// Get all model downloads
    pub fn all_models(&self) -> &HashMap<String, ComponentDownload> {
        &self.config.models
    }

    /// Resolve a URL to either a cached file path or the original URL
    ///
    /// This is the main method to use when downloading. It will:
    /// 1. Extract filename from URL
    /// 2. Check if file exists in cache
    /// 3. Return cached path if available, otherwise return original URL
    ///
    /// # Arguments
    /// * `url` - The download URL
    ///
    /// # Returns
    /// * `CacheResult` - Either a cached file path or the URL to download from
    pub fn resolve_url(&self, url: &str) -> CacheResult {
        let filename = Self::extract_filename(url);

        if let Some(cached_path) = self.get_cached_path(&filename) {
            info!("Using cached file: {:?}", cached_path);
            CacheResult::Cached(cached_path)
        } else {
            trace!("File not in cache, will download: {}", url);
            CacheResult::Download {
                url: url.to_string(),
                cache_path: self.get_cache_path(&filename),
            }
        }
    }

    /// Resolve a URL for a specific component
    ///
    /// Uses the filename from config if available, otherwise extracts from URL
    ///
    /// # Arguments
    /// * `component` - Component name
    /// * `url` - Fallback URL if component not in config
    pub fn resolve_component_url(&self, component: &str, url: &str) -> CacheResult {
        // Check if we have config for this component
        if let Some(comp) = self.get_component(component) {
            let cached_path = self.cache_dir.join(&comp.filename);
            if cached_path.exists()
                && fs::metadata(&cached_path)
                    .map(|m| m.len() > 0)
                    .unwrap_or(false)
            {
                info!("Using cached {} from: {:?}", comp.name, cached_path);
                return CacheResult::Cached(cached_path);
            }
            // Use URL from config
            trace!("Will download {} from config URL", comp.name);
            return CacheResult::Download {
                url: comp.url.clone(),
                cache_path: self.cache_dir.join(&comp.filename),
            };
        }

        // Fall back to URL-based resolution
        self.resolve_url(url)
    }

    /// Save a downloaded file to the cache
    ///
    /// # Arguments
    /// * `source` - Path to the downloaded file
    /// * `filename` - Filename to use in the cache
    pub fn save_to_cache(&self, source: &Path, filename: &str) -> Result<PathBuf> {
        let cache_path = self.cache_dir.join(filename);

        // If source is already in the cache directory, just return it
        if source == cache_path {
            return Ok(cache_path);
        }

        // Copy to cache
        fs::copy(source, &cache_path)
            .with_context(|| format!("Failed to copy {:?} to cache at {:?}", source, cache_path))?;

        info!("Cached file: {:?}", cache_path);
        Ok(cache_path)
    }

    /// Extract filename from a URL
    pub fn extract_filename(url: &str) -> String {
        url.split('/')
            .last()
            .unwrap_or("download.tmp")
            .split('?')
            .next()
            .unwrap_or("download.tmp")
            .to_string()
    }

    /// Verify a cached file's checksum if sha256 is provided
    ///
    /// # Arguments
    /// * `filename` - The cached filename
    /// * `expected_sha256` - Expected SHA256 hash (empty string to skip)
    pub fn verify_checksum(&self, filename: &str, expected_sha256: &str) -> Result<bool> {
        if expected_sha256.is_empty() {
            return Ok(true); // Skip verification if no hash provided
        }

        let cached_path = self.cache_dir.join(filename);
        if !cached_path.exists() {
            return Ok(false);
        }

        let content = fs::read(&cached_path)?;
        let computed = sha256_hex(&content);

        if computed == expected_sha256.to_lowercase() {
            trace!("Checksum verified for {}", filename);
            Ok(true)
        } else {
            warn!(
                "Checksum mismatch for {}: expected {}, got {}",
                filename, expected_sha256, computed
            );
            Ok(false)
        }
    }

    /// List all cached files
    pub fn list_cached(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        files.push(name.to_string());
                    }
                }
            }
        }

        files.sort();
        Ok(files)
    }

    /// Get total size of cached files in bytes
    pub fn cache_size(&self) -> Result<u64> {
        let mut total = 0u64;

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    total += entry.metadata()?.len();
                }
            }
        }

        Ok(total)
    }

    /// Clear all cached files
    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    fs::remove_file(entry.path())?;
                }
            }
            info!("Cleared cache directory: {:?}", self.cache_dir);
        }
        Ok(())
    }
}

/// Result of resolving a URL through the cache
#[derive(Debug)]
pub enum CacheResult {
    /// File was found in cache
    Cached(PathBuf),
    /// File needs to be downloaded
    Download {
        /// URL to download from
        url: String,
        /// Path where file should be cached
        cache_path: PathBuf,
    },
}

impl CacheResult {
    /// Check if result is a cached file
    pub fn is_cached(&self) -> bool {
        matches!(self, CacheResult::Cached(_))
    }

    /// Get the path (either cached or target cache path)
    pub fn path(&self) -> &Path {
        match self {
            CacheResult::Cached(p) => p,
            CacheResult::Download { cache_path, .. } => cache_path,
        }
    }

    /// Get the URL if this is a download result
    pub fn url(&self) -> Option<&str> {
        match self {
            CacheResult::Cached(_) => None,
            CacheResult::Download { url, .. } => Some(url),
        }
    }
}

/// Compute SHA256 hash of data and return as lowercase hex string
fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_config(dir: &Path) -> Result<()> {
        let config = r#"
[cache_settings]
cache_dir = "test-cache"

[components.test]
name = "Test Component"
url = "https://example.com/test.tar.gz"
filename = "test.tar.gz"
sha256 = ""

[models.test_model]
name = "Test Model"
url = "https://example.com/model.gguf"
filename = "model.gguf"
sha256 = ""
"#;
        let config_path = dir.join(CONFIG_FILE);
        fs::write(config_path, config)?;
        Ok(())
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(
            DownloadCache::extract_filename("https://example.com/path/file.tar.gz"),
            "file.tar.gz"
        );
        assert_eq!(
            DownloadCache::extract_filename("https://example.com/file.zip?token=abc"),
            "file.zip"
        );
        assert_eq!(DownloadCache::extract_filename("https://example.com/"), "");
    }

    #[test]
    fn test_cache_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(temp_dir.path())?;

        let cache = DownloadCache::new(temp_dir.path())?;

        assert!(cache.cache_dir().exists());
        assert_eq!(cache.cache_dir().file_name().unwrap(), "test-cache");

        Ok(())
    }

    #[test]
    fn test_is_cached() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(temp_dir.path())?;

        let cache = DownloadCache::new(temp_dir.path())?;

        // Initially not cached
        assert!(!cache.is_cached("test.tar.gz"));

        // Create a cached file
        let cache_path = cache.get_cache_path("test.tar.gz");
        let mut file = fs::File::create(&cache_path)?;
        file.write_all(b"test content")?;

        // Now it should be cached
        assert!(cache.is_cached("test.tar.gz"));

        // Empty file should not count as cached
        let empty_path = cache.get_cache_path("empty.tar.gz");
        fs::File::create(&empty_path)?;
        assert!(!cache.is_cached("empty.tar.gz"));

        Ok(())
    }

    #[test]
    fn test_resolve_url() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(temp_dir.path())?;

        let cache = DownloadCache::new(temp_dir.path())?;

        // Test with uncached URL
        let result = cache.resolve_url("https://example.com/newfile.tar.gz");
        assert!(!result.is_cached());
        assert_eq!(result.url(), Some("https://example.com/newfile.tar.gz"));

        // Create cached file
        let cache_path = cache.get_cache_path("newfile.tar.gz");
        let mut file = fs::File::create(&cache_path)?;
        file.write_all(b"cached content")?;

        // Now it should resolve to cached
        let result = cache.resolve_url("https://example.com/newfile.tar.gz");
        assert!(result.is_cached());
        assert!(result.url().is_none());

        Ok(())
    }

    #[test]
    fn test_get_component() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(temp_dir.path())?;

        let cache = DownloadCache::new(temp_dir.path())?;

        let component = cache.get_component("test");
        assert!(component.is_some());
        assert_eq!(component.unwrap().name, "Test Component");

        let missing = cache.get_component("nonexistent");
        assert!(missing.is_none());

        Ok(())
    }

    #[test]
    fn test_list_cached() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(temp_dir.path())?;

        let cache = DownloadCache::new(temp_dir.path())?;

        // Create some cached files
        fs::write(cache.get_cache_path("file1.tar.gz"), "content1")?;
        fs::write(cache.get_cache_path("file2.zip"), "content2")?;

        let files = cache.list_cached()?;
        assert_eq!(files.len(), 2);
        assert!(files.contains(&"file1.tar.gz".to_string()));
        assert!(files.contains(&"file2.zip".to_string()));

        Ok(())
    }

    #[test]
    fn test_cache_size() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(temp_dir.path())?;

        let cache = DownloadCache::new(temp_dir.path())?;

        // Initially empty
        assert_eq!(cache.cache_size()?, 0);

        // Add files
        fs::write(cache.get_cache_path("file1.txt"), "12345")?; // 5 bytes
        fs::write(cache.get_cache_path("file2.txt"), "1234567890")?; // 10 bytes

        assert_eq!(cache.cache_size()?, 15);

        Ok(())
    }

    #[test]
    fn test_clear_cache() -> Result<()> {
        let temp_dir = TempDir::new()?;
        create_test_config(temp_dir.path())?;

        let cache = DownloadCache::new(temp_dir.path())?;

        // Create some cached files
        fs::write(cache.get_cache_path("file1.tar.gz"), "content1")?;
        fs::write(cache.get_cache_path("file2.zip"), "content2")?;

        assert_eq!(cache.list_cached()?.len(), 2);

        cache.clear_cache()?;

        assert_eq!(cache.list_cached()?.len(), 0);

        Ok(())
    }
}
