use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::test::{Test, TestConfig};

#[derive(Debug, Serialize, Clone)]
pub struct Group {
    pub name: String,
    pub tests: Vec<Test>,
    pub desc: String,
}

impl Group {
    pub fn from_config(group_config: &GroupConfig, group_id: &str) -> Self {
        Self {
            name: group_id.to_string(),
            tests: group_config
                .tests
                .iter()
                .map(|(id, t)| Test::from_config(t, id))
                .collect(),
            desc: group_config.desc.clone().unwrap_or("".to_string()),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GroupConfig {
    pub desc: Option<String>,

    #[serde(rename = "test")]
    pub tests: IndexMap<String, TestConfig>,
}
