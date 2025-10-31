use anyhow::{Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::target::TargetConfig;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RunnerConfig {
    pub command: Option<String>,
    pub fail_regex_template: Option<String>,
    pub pass_regex_template: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub work_dir: Option<PathBuf>,
}

#[derive(Clone)]
pub struct DefaultRunnerConfig {
    id: String,
    config: RunnerConfig,
}

impl DefaultRunnerConfig {
    pub fn into_pair(self) -> (String, RunnerConfig) {
        (self.id, self.config)
    }
}

pub enum DefaultRunner {
    Pytest,
    BunTest,
    Vitest,
    XcodebuildMacOS,
}

impl DefaultRunner {
    pub fn get_default_config(&self, target_config: &TargetConfig) -> DefaultRunnerConfig {
        match self {
            DefaultRunner::Pytest => DefaultRunnerConfig {
                id: "pytest -v".to_string(),
                config: RunnerConfig {
                    env: None,
                    command: Some("pytest -v".to_string()),
                    fail_regex_template: Some(
                        "{{ file_name }}::test_{{ test_name | convert_case('Snake') }} FAILED"
                            .to_string(),
                    ),
                    pass_regex_template: Some(
                        "{{ file_name }}::test_{{ test_name | convert_case('Snake') }} PASSED"
                            .to_string(),
                    ),
                    work_dir: Some(target_config.out_dir.clone()),
                },
            },
            DefaultRunner::BunTest => DefaultRunnerConfig {
                id: "bun test".to_string(),
                config: RunnerConfig {
                    env: None,
                    command: Some("bun test".to_string()),
                    fail_regex_template: Some(
                        r"\(fail\) {{ suite_name }} > {{ group_name }} > {{ test_name }}( \[\d+\.\d+ms])*$"
                            .to_string(),
                    ),
                    pass_regex_template: Some(
                        r"\(pass\) {{ suite_name }} > {{ group_name }} > {{ test_name }}( \[\d+\.\d+ms])*$"
                            .to_string(),
                    ),
                    work_dir: Some(target_config.out_dir.clone()),
                },
            },
            DefaultRunner::Vitest => DefaultRunnerConfig {
                id: "vitest".to_string(),
                config: RunnerConfig {
                    env: None,
                    command: Some("npx vitest run --no-color --reporter verbose".to_string()),
                    fail_regex_template: Some(
                        "FAIL  {{ file_name }} > {{ suite_name }} > {{ group_name }} > {{ test_name }}"
                            .to_string(),
                    ),
                    pass_regex_template: Some(
                        "✓ {{ file_name }} > {{ suite_name }} > {{ group_name }} > {{ test_name }}"
                            .to_string(),
                    ),
                    work_dir: Some(target_config.out_dir.clone()),
                },
            },
            DefaultRunner::XcodebuildMacOS => {
                let work_dir = target_config
                    .out_dir
                    .parent()
                    .and_then(|p| p.parent())
                    .expect("parent should always exist")
                    .to_path_buf();

                DefaultRunnerConfig {
                    id: "macOS".to_string(),
                    config: RunnerConfig {
                        env: None,
                        command: Some(
                            r#"xcodebuild -scheme {{ package_name | convert_case('Pascal') }} test -destination "platform=macOS""#
                                .to_string(),
                        ),
                        pass_regex_template: Some(
                            r#""{{ suite_name }}: {{ test_name }}" passed"#.to_string(),
                        ),
                        fail_regex_template: Some(
                            r#"Failing tests:(.|\W)*{{ (suite_name + " " + test_name) | convert_case('Camel') }}\(\)(.|\W)*** TEST FAILED **"#
                                .to_string(),
                        ),
                        work_dir: Some(work_dir),
                    },
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Runner {
    pub command: String,
    pub fail_regex_template: String,
    pub pass_regex_template: String,
    pub env: Option<HashMap<String, String>>,
    pub work_dir: PathBuf,
}

impl Runner {
    pub fn from_configs(
        default_configs: IndexMap<String, RunnerConfig>,
        configs: &IndexMap<String, RunnerConfig>,
        out_dir: &Path,
    ) -> Result<IndexMap<String, Self>> {
        let mut current_cfg = RunnerConfig::default();

        let mut runners: IndexMap<String, Self> = IndexMap::new();

        default_configs
            .iter()
            .chain(configs.iter())
            .try_for_each(|(id, cfg)| {
                current_cfg = RunnerConfig {
                    command: cfg.command.clone().or_else(|| current_cfg.command.clone()),
                    env: cfg.env.clone().or_else(|| current_cfg.env.clone()),
                    work_dir: cfg
                        .work_dir
                        .clone()
                        .or_else(|| current_cfg.work_dir.clone()),
                    fail_regex_template: cfg
                        .fail_regex_template
                        .clone()
                        .or_else(|| current_cfg.fail_regex_template.clone()),
                    pass_regex_template: cfg
                        .pass_regex_template
                        .clone()
                        .or_else(|| current_cfg.pass_regex_template.clone()),
                };

                let runner = Runner {
                    command: current_cfg
                        .command
                        .clone()
                        .context(format!("command not defined for runner: {}", id))?,
                    fail_regex_template: "(?m)".to_owned()
                        + &current_cfg.fail_regex_template.clone().context(format!(
                            "fail_regex_template not defined for runner: {}",
                            id
                        ))?,
                    pass_regex_template: "(?m)".to_owned()
                        + &current_cfg.pass_regex_template.clone().context(format!(
                            "pass_regex_template not defined for runner: {}",
                            id
                        ))?,
                    env: current_cfg.env.clone(),
                    work_dir: current_cfg
                        .work_dir
                        .clone()
                        .unwrap_or_else(|| out_dir.to_owned()),
                };

                runners.insert(id.clone(), runner);

                Ok::<(), anyhow::Error>(())
            })?;

        Ok(runners)
    }
}
