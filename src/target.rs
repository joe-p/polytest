use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::find_template_file;
use crate::runner::DefaultRunner;
use crate::runner::Runner;
use crate::runner::RunnerConfig;
use crate::TemplateType;

pub enum DefaultTarget {
    Pytest,
    Bun,
    Vitest,
    Swift,
}

impl TryFrom<&str> for DefaultTarget {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pytest" => Ok(DefaultTarget::Pytest),
            "bun" => Ok(DefaultTarget::Bun),
            "vitest" => Ok(DefaultTarget::Vitest),
            "swift" => Ok(DefaultTarget::Swift),
            _ => Err(anyhow!("Unsupported default target: {}", value)),
        }
    }
}

impl Display for DefaultTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DefaultTarget::Pytest => "pytest",
            DefaultTarget::Bun => "bun",
            DefaultTarget::Vitest => "vitest",
            DefaultTarget::Swift => "swift",
        };
        write!(f, "{}", s)
    }
}

impl DefaultTarget {
    fn default_runners(&self) -> Vec<DefaultRunner> {
        match self {
            DefaultTarget::Pytest => vec![DefaultRunner::Pytest],
            DefaultTarget::Bun => vec![DefaultRunner::BunTest],
            DefaultTarget::Vitest => vec![DefaultRunner::Vitest],
            DefaultTarget::Swift => vec![DefaultRunner::XcodebuildMacOS],
        }
    }

    pub fn build_target(
        &self,
        id: &str,
        config: &TargetConfig,
        config_root: &Path,
    ) -> Result<Target> {
        let runner_overrides = config.runners.clone().unwrap_or_default();

        match self {
            DefaultTarget::Pytest => {
                let target_out_dir = config_root.join(&config.out_dir);
                let default_runner_cfgs: IndexMap<String, RunnerConfig> = self
                    .default_runners()
                    .into_iter()
                    .map(|default_runner| default_runner.get_default_config(config).into_pair())
                    .collect();

                let runners =
                    Runner::from_configs(default_runner_cfgs, &runner_overrides, &target_out_dir)?;

                Ok(Target {
                    id: id.to_string(),
                    test_regex_template: r"(?m)def test_{{ name | convert_case('Snake') }}\("
                        .to_string(),
                    suite_file_name_template: "test_{{ suite.name | convert_case('Snake') }}.py"
                        .to_string(),
                    out_dir: target_out_dir,
                    suite_template: self.get_template_content(TemplateType::Suite),
                    group_template: self.get_template_content(TemplateType::Group),
                    test_template: self.get_template_content(TemplateType::Test),
                    runners,
                })
            }
            DefaultTarget::Bun => {
                let target_out_dir = config_root.join(&config.out_dir);
                let default_runner_cfgs: IndexMap<String, RunnerConfig> = self
                    .default_runners()
                    .into_iter()
                    .map(|default_runner| default_runner.get_default_config(config).into_pair())
                    .collect();

                let runners =
                    Runner::from_configs(default_runner_cfgs, &runner_overrides, &target_out_dir)?;

                Ok(Target {
                    id: id.to_string(),
                    test_regex_template: r#"(?m)test\("{{ name }}","#.to_string(),
                    suite_file_name_template: "{{ suite.name | convert_case('Snake') }}.test.ts"
                        .to_string(),
                    out_dir: target_out_dir,
                    suite_template: self.get_template_content(TemplateType::Suite),
                    group_template: self.get_template_content(TemplateType::Group),
                    test_template: self.get_template_content(TemplateType::Test),
                    runners,
                })
            }
            DefaultTarget::Vitest => {
                let target_out_dir = config_root.join(&config.out_dir);
                let default_runner_cfgs: IndexMap<String, RunnerConfig> = self
                    .default_runners()
                    .into_iter()
                    .map(|default_runner| default_runner.get_default_config(config).into_pair())
                    .collect();

                let runners =
                    Runner::from_configs(default_runner_cfgs, &runner_overrides, &target_out_dir)?;

                Ok(Target {
                    id: id.to_string(),
                    test_regex_template: r#"(?m)test\("{{ name }}","#.to_string(),
                    suite_file_name_template: "{{ suite.name | convert_case('Snake') }}.test.ts"
                        .to_string(),
                    out_dir: target_out_dir,
                    suite_template: self.get_template_content(TemplateType::Suite),
                    group_template: self.get_template_content(TemplateType::Group),
                    test_template: self.get_template_content(TemplateType::Test),
                    runners,
                })
            }
            DefaultTarget::Swift => {
                let target_out_dir = config_root.join(&config.out_dir);
                let default_runner_cfgs: IndexMap<String, RunnerConfig> = self
                    .default_runners()
                    .into_iter()
                    .map(|default_runner| default_runner.get_default_config(config).into_pair())
                    .collect();

                let runners =
                    Runner::from_configs(default_runner_cfgs, &runner_overrides, &target_out_dir)?;

                Ok(Target {
                    id: id.to_string(),
                    test_regex_template: r#"(?m)@Test\(".+: {{ name }}""#.to_string(),
                    suite_file_name_template:
                        "{{ suite.name | convert_case('Pascal') }}Tests.swift".to_string(),
                    out_dir: target_out_dir,
                    suite_template: self.get_template_content(TemplateType::Suite),
                    group_template: self.get_template_content(TemplateType::Group),
                    test_template: self.get_template_content(TemplateType::Test),
                    runners,
                })
            }
        }
    }

    fn get_template_content(&self, tmpl_type: TemplateType) -> String {
        (match self {
            DefaultTarget::Pytest => match tmpl_type {
                TemplateType::Suite => include_str!("../templates/pytest/suite.py.jinja"),
                TemplateType::Group => include_str!("../templates/pytest/group.py.jinja"),
                TemplateType::Test => include_str!("../templates/pytest/test.py.jinja"),
            },
            DefaultTarget::Bun => match tmpl_type {
                TemplateType::Suite => include_str!("../templates/bun/suite.ts.jinja"),
                TemplateType::Group => include_str!("../templates/bun/group.ts.jinja"),
                TemplateType::Test => include_str!("../templates/bun/test.ts.jinja"),
            },
            DefaultTarget::Vitest => match tmpl_type {
                TemplateType::Suite => include_str!("../templates/vitest/suite.ts.jinja"),
                TemplateType::Group => include_str!("../templates/vitest/group.ts.jinja"),
                TemplateType::Test => include_str!("../templates/vitest/test.ts.jinja"),
            },
            DefaultTarget::Swift => match tmpl_type {
                TemplateType::Suite => include_str!("../templates/swift/suite.swift.jinja"),
                TemplateType::Group => include_str!("../templates/swift/group.swift.jinja"),
                TemplateType::Test => include_str!("../templates/swift/test.swift.jinja"),
            },
        })
        .to_string()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Target {
    pub id: String,
    pub out_dir: PathBuf,
    pub test_regex_template: String,
    pub suite_file_name_template: String,
    pub suite_template: String,
    pub group_template: String,
    pub test_template: String,
    pub runners: IndexMap<String, Runner>,
}

impl Target {
    pub fn from_config(config: &TargetConfig, id: &str, config_root: &Path) -> Result<Self> {
        let default_target = DefaultTarget::try_from(id)?;
        default_target.build_target(id, config, config_root)
    }

    pub fn from_custom_config(
        config: &CustomTargetConfig,
        id: &str,
        config_root: &Path,
    ) -> Result<Self> {
        let template_dir = &config_root.join(&config.template_dir);

        let suite_file = find_template_file(template_dir, "suite*")
            .context(format!("failed to find suite template for {}", id))?;
        let suite_template = std::fs::read_to_string(suite_file)
            .context(format!("failed to read suite template file for {}", id))?;

        let group_file = find_template_file(template_dir, "group*")
            .context(format!("failed to find group template for {}", id))?;
        let group_template = std::fs::read_to_string(group_file)
            .context(format!("failed to read group template file for {}", id))?;

        let test_file = find_template_file(template_dir, "test*")
            .context(format!("failed to find test template for {}", id))?;
        let test_template = std::fs::read_to_string(test_file)
            .context(format!("failed to read test template file for {}", id))?;

        Ok(Self {
            id: id.to_string(),
            test_regex_template: "(?m)".to_owned() + config.test_regex_template.as_str(),
            out_dir: config_root.join(&config.out_dir),
            suite_file_name_template: config.suite_file_name_template.clone(),
            suite_template,
            group_template,
            test_template,
            runners: Runner::from_configs(
                IndexMap::<String, RunnerConfig>::default(),
                &config.runners,
                &config_root.join(&config.out_dir),
            )?,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct TargetConfig {
    pub out_dir: PathBuf,

    #[serde(rename = "runner")]
    pub runners: Option<IndexMap<String, RunnerConfig>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomTargetConfig {
    out_dir: PathBuf,

    test_regex_template: String,
    suite_file_name_template: String,
    template_dir: PathBuf,

    #[serde(rename = "runner")]
    runners: IndexMap<String, RunnerConfig>,
}

impl From<Target> for CustomTargetConfig {
    fn from(target: Target) -> Self {
        Self {
            out_dir: target.out_dir,
            test_regex_template: target.test_regex_template,
            suite_file_name_template: target.suite_file_name_template,
            template_dir: PathBuf::from(""),
            runners: target
                .runners
                .into_iter()
                .map(|(id, runner)| {
                    (
                        id,
                        RunnerConfig {
                            command: Some(runner.command),
                            fail_regex_template: Some(runner.fail_regex_template),
                            pass_regex_template: Some(runner.pass_regex_template),
                            env: runner.env,
                            work_dir: Some(runner.work_dir),
                        },
                    )
                })
                .collect(),
        }
    }
}
