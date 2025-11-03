use color_eyre::eyre::{Context, Result};
use indexmap::IndexMap;
use json_comments::StripComments;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

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

        let config: Config = if path.ends_with(".json") {
            let stripped = StripComments::new(contents.as_bytes());

            serde_json::from_reader(stripped).context("failed to parse config file")?
        } else {
            toml::from_str(&contents).context("failed to parse config file")?
        };
        Ok(Self {
            root_dir: PathBuf::from(path)
                .parent()
                .unwrap_or(PathBuf::from(".").as_path())
                .to_path_buf(),
            config,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub name: String,

    pub package_name: String,

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
