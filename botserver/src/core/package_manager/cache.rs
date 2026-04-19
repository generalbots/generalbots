use anyhow::{Context, Result};
use log::{debug, info, trace, warn};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_CACHE_DIR: &str = "./botserver-installers";

const CONFIG_FILE: &str = "3rdparty.toml";

#[derive(Debug, Deserialize, Default)]
pub struct ThirdPartyConfig {
    #[serde(default)]
    pub cache_settings: CacheSettings,
    #[serde(default)]
    pub components: HashMap<String, ComponentDownload>,
    #[serde(default)]
    pub models: HashMap<String, ComponentDownload>,
}

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

#[derive(Debug, Deserialize, Clone)]
pub struct ComponentDownload {
    pub name: String,
    pub url: String,
    pub filename: String,
    #[serde(default)]
    pub sha256: String,
}

#[derive(Debug)]
pub struct DownloadCache {
    base_path: PathBuf,

    cache_dir: PathBuf,

    config: ThirdPartyConfig,
}

impl DownloadCache {
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        let config = Self::load_config(&base_path)?;

        let cache_dir = if let Ok(installers_path) = std::env::var("BOTSERVER_INSTALLERS_PATH") {
            let path = PathBuf::from(&installers_path);
            if path.exists() {
                info!(
                    "Using installers from BOTSERVER_INSTALLERS_PATH: {}",
                    path.display()
                );
                path
            } else {
                warn!(
                    "BOTSERVER_INSTALLERS_PATH set but path doesn't exist: {}",
                    path.display()
                );
                base_path.join(&config.cache_settings.cache_dir)
            }
        } else {
            base_path.join(&config.cache_settings.cache_dir)
        };

        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).with_context(|| {
                format!("Failed to create cache directory: {}", cache_dir.display())
            })?;
            info!("Created cache directory: {}", cache_dir.display());
        }

        Ok(Self {
            base_path,
            cache_dir,
            config,
        })
    }

    fn load_config(base_path: &Path) -> Result<ThirdPartyConfig> {
        let config_path = base_path.join(CONFIG_FILE);

        if !config_path.exists() {
            debug!(
                "No {} found at {}, using embedded configuration",
                CONFIG_FILE,
                config_path.display()
            );
            let content = include_str!("../../../3rdparty.toml");
            let config: ThirdPartyConfig =
                toml::from_str(content).with_context(|| "Failed to parse embedded config file")?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: ThirdPartyConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

        debug!(
            "Loaded {} with {} components and {} models",
            CONFIG_FILE,
            config.components.len(),
            config.models.len()
        );

        Ok(config)
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    pub fn is_cached(&self, filename: &str) -> bool {
        let cached_path = self.cache_dir.join(filename);
        if cached_path.exists() {
            if let Ok(metadata) = fs::metadata(&cached_path) {
                return metadata.len() > 0;
            }
        }
        false
    }

    pub fn get_cached_path(&self, filename: &str) -> Option<PathBuf> {
        let cached_path = self.cache_dir.join(filename);
        if self.is_cached(filename) {
            Some(cached_path)
        } else {
            None
        }
    }

    pub fn get_cache_path(&self, filename: &str) -> PathBuf {
        self.cache_dir.join(filename)
    }

    pub fn get_component(&self, component: &str) -> Option<&ComponentDownload> {
        self.config.components.get(component)
    }

    pub fn get_model(&self, model: &str) -> Option<&ComponentDownload> {
        self.config.models.get(model)
    }

    pub fn all_components(&self) -> &HashMap<String, ComponentDownload> {
        &self.config.components
    }

    pub fn all_models(&self) -> &HashMap<String, ComponentDownload> {
        &self.config.models
    }

    pub fn resolve_url(&self, url: &str) -> CacheResult {
        let filename = Self::extract_filename(url);

        if let Some(cached_path) = self.get_cached_path(&filename) {
            info!("Using cached file: {}", cached_path.display());
            CacheResult::Cached(cached_path)
        } else {
            trace!("File not in cache, will download: {}", url);
            CacheResult::Download {
                url: url.to_string(),
                cache_path: self.get_cache_path(&filename),
            }
        }
    }

    pub fn resolve_component_url(&self, component: &str, url: &str) -> CacheResult {
        if let Some(comp) = self.get_component(component) {
            let cached_path = self.cache_dir.join(&comp.filename);
            if cached_path.exists()
                && fs::metadata(&cached_path)
                    .map(|m| m.len() > 0)
                    .unwrap_or(false)
            {
                info!("Using cached {} from: {}", comp.name, cached_path.display());
                return CacheResult::Cached(cached_path);
            }

            trace!("Will download {} from config URL", comp.name);
            return CacheResult::Download {
                url: comp.url.clone(),
                cache_path: self.cache_dir.join(&comp.filename),
            };
        }

        self.resolve_url(url)
    }

    pub fn save_to_cache(&self, source: &Path, filename: &str) -> Result<PathBuf> {
        let cache_path = self.cache_dir.join(filename);

        if source == cache_path {
            return Ok(cache_path);
        }

        fs::copy(source, &cache_path).with_context(|| {
            format!(
                "Failed to copy {} to cache at {}",
                source.display(),
                cache_path.display()
            )
        })?;

        info!("Cached file: {}", cache_path.display());
        Ok(cache_path)
    }

    pub fn extract_filename(url: &str) -> String {
        url.split('/')
            .next_back()
            .unwrap_or("download.tmp")
            .split('?')
            .next()
            .unwrap_or("download.tmp")
            .to_string()
    }

    pub fn verify_checksum(&self, filename: &str, expected_sha256: &str) -> Result<bool> {
        if expected_sha256.is_empty() {
            return Ok(true);
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

    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    fs::remove_file(entry.path())?;
                }
            }
            info!("Cleared cache directory: {}", self.cache_dir.display());
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum CacheResult {
    Cached(PathBuf),

    Download { url: String, cache_path: PathBuf },
}

impl CacheResult {
    pub fn is_cached(&self) -> bool {
        matches!(self, Self::Cached(_))
    }

    pub fn path(&self) -> &Path {
        match self {
            Self::Cached(p) => p,
            Self::Download { cache_path, .. } => cache_path,
        }
    }

    pub fn url(&self) -> Option<&str> {
        match self {
            Self::Cached(_) => None,
            Self::Download { url, .. } => Some(url),
        }
    }
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}
