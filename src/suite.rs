use serde::{Deserialize, Serialize};

use crate::{config::Config, group::Group};

#[derive(Deserialize, Debug, Clone)]
pub struct SuiteConfig {
    groups: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Suite {
    pub name: String,
    pub groups: Vec<Group>,
}

impl Suite {
    pub fn from_config(config: &Config, suite_config: &SuiteConfig, suite_id: &str) -> Self {
        Self {
            name: suite_id.to_string(),
            groups: config
                .groups
                .iter()
                .filter_map(|(id, g)| {
                    if suite_config.groups.contains(id) {
                        Some(Group::from_config(g, id))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}
