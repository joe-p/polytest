use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct TestConfig {
    #[serde(default)]
    exclude_targets: Vec<String>,
    desc: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Test {
    pub name: String,
    pub desc: String,
    pub exclude_targets: Vec<String>,
}

impl Test {
    pub fn from_config(test_config: &TestConfig, test_id: &str) -> Self {
        Self {
            exclude_targets: test_config.exclude_targets.clone(),
            name: test_id.to_string(),
            desc: test_config.desc.clone().unwrap_or("".to_string()),
        }
    }
}
