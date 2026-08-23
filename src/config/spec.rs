use serde::{Deserialize, Serialize};
use std::{borrow::Cow, path::PathBuf};

/// Root configuration file for Meriadoc (~/.config/meriadoc/config.yaml)
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MeriadocConfig {
    /// Paths where projects can be discovered
    #[serde(default)]
    pub discovery: DiscoveryConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Audit logging configuration
    #[serde(default)]
    pub audit: AuditConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoveryConfig {
    /// List of root directories to search for meriadoc.yaml
    pub roots: Vec<DiscoveryRoot>,

    /// Maximum directory depth when searching for specs
    pub max_depth: usize,

    /// Whether discovery should validate specs immediately
    pub validate_on_discovery: bool,

    /// Names of specfiles accepted
    pub spec_files: Vec<Cow<'static, str>>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            max_depth: 3,
            validate_on_discovery: true,
            spec_files: vec![
                Cow::Borrowed("meriadoc.yaml"),
                Cow::Borrowed("meriadoc.yml"),
                Cow::Borrowed("merry.yaml"),
                Cow::Borrowed("merry.yml"),
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryRoot {
    /// Root path for discovery
    pub path: PathBuf,

    /// Optional human-friendly name
    pub name: Option<String>,

    /// Whether this root is currently enabled
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Enable or disable cache entirely
    pub enabled: bool,

    /// Directory where cached specs and metadata are stored
    pub dir: PathBuf,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            // Sentinel: ConfigLoader replaces this with the absolute path at load time.
            dir: PathBuf::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditConfig {
    /// Audit logging is off by default to avoid surprising existing users.
    pub enabled: bool,

    /// Sinks to write audit events to.
    pub sinks: Vec<AuditSinkConfig>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            // Sentinel: ConfigLoader replaces empty path with the absolute default.
            sinks: vec![AuditSinkConfig::File {
                path: PathBuf::new(),
            }],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuditSinkConfig {
    File { path: PathBuf },
    Stderr,
}
