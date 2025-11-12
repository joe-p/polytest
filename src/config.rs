use color_eyre::eyre::{bail, Context, Result};
use indexmap::IndexMap;
use json_comments::StripComments;
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

static VALID_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_\- ]+$").unwrap());

use crate::{
    document::DocumentConfig,
    group::GroupConfig,
    suite::SuiteConfig,
    target::{CustomTargetConfig, TargetConfig},
};

#[derive(Clone)]
pub struct ConfigMeta {
    pub root_dir: PathBuf,
    pub config: Config,
}

impl ConfigMeta {
    pub fn from_file(path: &str) -> Result<Self> {
        let full_path = std::fs::canonicalize(path)
            .with_context(|| format!("failed to canonicalize path: {}", path))?;

        let contents = std::fs::read_to_string(path).context(format!(
            "failed to read config file: {}",
            full_path.display()
        ))?;

        let stripped = StripComments::new(contents.as_bytes());

        let config: Config =
            serde_json::from_reader(stripped).context("failed to parse config file")?;

        // Validate version is present and MAJOR.MINOR matches Cargo.toml
        const CARGO_VERSION: &str = env!("CARGO_PKG_VERSION");
        match &config.version {
            None => {
                let cargo_major_minor = CARGO_VERSION
                    .split('.')
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(".");
                bail!(
                    "Config is missing required 'version' field. Please add '\"version\": \"{}\"' to your polytest.json file in MAJOR.MINOR format.",
                    cargo_major_minor
                );
            }
            Some(config_version) => {
                // Enforce that config version is MAJOR.MINOR format only
                let version_parts: Vec<&str> = config_version.split('.').collect();
                if version_parts.len() != 2 {
                    bail!(
                        "Config version must be in MAJOR.MINOR format (e.g., \"0.4\"), but found '{}' with {} components. Please remove the PATCH version.",
                        config_version,
                        version_parts.len()
                    );
                }

                // Extract MAJOR.MINOR from cargo version
                let cargo_major_minor = CARGO_VERSION
                    .split('.')
                    .take(2)
                    .collect::<Vec<_>>()
                    .join(".");

                if config_version != &cargo_major_minor {
                    bail!(
                        "Config version mismatch: polytest.json specifies version '{}' but polytest binary is version '{}' (MAJOR.MINOR: {}). Please update to '{}'.",
                        config_version,
                        CARGO_VERSION,
                        cargo_major_minor,
                        cargo_major_minor
                    );
                }
            }
        }

        let config_meta = Self {
            root_dir: PathBuf::from(path)
                .parent()
                .unwrap_or(PathBuf::from(".").as_path())
                .to_path_buf(),
            config,
        };

        // Validate test, suite, and group names
        config_meta.validate_names()?;

        Ok(config_meta)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub name: String,

    pub package_name: String,

    pub version: Option<String>,

    #[serde(default)]
    pub resource_dir: Option<PathBuf>,

    #[serde(rename = "document")]
    #[serde(default)]
    pub documents: HashMap<String, DocumentConfig>,

    #[serde(rename = "target")]
    #[serde(default)]
    pub targets: HashMap<String, TargetConfig>,

    #[serde(rename = "custom_target")]
    #[serde(default)]
    pub custom_targets: HashMap<String, CustomTargetConfig>,

    #[serde(rename = "suite")]
    pub suites: IndexMap<String, SuiteConfig>,

    #[serde(rename = "group")]
    pub groups: IndexMap<String, GroupConfig>,
}

impl ConfigMeta {
    /// Validates that a name contains only alphanumeric characters, underscores, dashes, and spaces
    fn validate_name(name: &str, name_type: &str) -> Result<()> {
        if name.is_empty() {
            bail!("{} name cannot be empty", name_type);
        }

        // Maybe we just wanr on leading/trailing, but for now we'll error because I can't think of a reason
        // to intentionally want this
        if name.starts_with(" ") || name.ends_with(" ") {
            bail!("{} name cannot start or end with a space", name_type);
        }

        if !VALID_NAME_REGEX.is_match(name) {
            bail!(
                "{} name '{}' contains invalid characters. Only alphanumeric characters, underscores (_), dashes (-), and spaces are allowed.",
                name_type,
                name
            );
        }

        Ok(())
    }

    /// Validates all test, suite, and group names in the configuration
    fn validate_names(&self) -> Result<()> {
        // Validate suite names
        for suite_id in self.config.suites.keys() {
            Self::validate_name(suite_id, "Suite")?;
        }

        // Validate group names
        for group_id in self.config.groups.keys() {
            Self::validate_name(group_id, "Group")?;
        }

        // Validate test names within groups
        for (group_id, group_config) in &self.config.groups {
            for test_id in group_config.tests.keys() {
                Self::validate_name(test_id, &format!("Test (in group '{}')", group_id))?;
            }
        }

        Ok(())
    }
}
