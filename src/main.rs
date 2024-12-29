use anyhow::{anyhow, Context, Result};
use clap::{command, Args, Parser, Subcommand};
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn get_group_comment(group: &str) -> String {
    format!("Polytest Group: {}", group)
}

fn get_suite_comment(suite: &str) -> String {
    format!("Polytest Suite: {}", suite)
}

fn insert_after_keyword(original: &str, to_insert: &str, keyword: &str) -> String {
    match original.find(keyword) {
        Some(pos) => {
            let mut result = String::with_capacity(original.len() + to_insert.len());
            result.push_str(&original[..pos + keyword.len()]);
            result.push_str(to_insert);
            result.push_str(&original[pos + keyword.len()..]);
            result
        }
        None => panic!("Keyword not found: {}", keyword),
    }
}

fn case_from_str(s: &str) -> Result<Case> {
    match s {
        "Alternating" => Ok(Case::Alternating),
        "Camel" => Ok(Case::Camel),
        "Cobol" => Ok(Case::Cobol),
        "Flat" => Ok(Case::Flat),
        "Kebab" => Ok(Case::Kebab),
        "Lower" => Ok(Case::Lower),
        "Pascal" => Ok(Case::Pascal),
        "Snake" => Ok(Case::Snake),
        "ScreamingSnake" | "UpperSnake" => Ok(Case::UpperSnake),
        "Title" => Ok(Case::Title),
        "Toggle" => Ok(Case::Toggle),
        "Train" => Ok(Case::Train),
        "Upper" => Ok(Case::Upper),
        "UpperCamel" => Ok(Case::UpperCamel),
        "UpperFlat" => Ok(Case::UpperFlat),
        "UpperKebab" => Ok(Case::UpperKebab),
        _ => Err(anyhow!(
            "Unsupported case: {}. Supported cases are: Alternating, Camel, Cobol, Flat, Kebab, \
             Lower, Pascal, Snake, ScreamingSnake/UpperSnake, Title, Toggle, Train, Upper, \
             UpperCamel, UpperFlat, UpperKebab",
            s,
        )),
    }
}

pub fn convert_case_filter(input: &str, case: &str) -> String {
    input.to_case(case_from_str(case).unwrap_or_else(|e| {
        panic!("failed to convert case: {}", e);
    }))
}

pub struct ConfigMeta {
    root_dir: PathBuf,
    config: Config,
}

impl ConfigMeta {
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(path).context("failed to read config file")?;
        let config = toml::from_str(&contents).context("failed to parse config file")?;
        Ok(Self {
            root_dir: PathBuf::from(path)
                .parent()
                .unwrap_or(PathBuf::from(".").as_path())
                .to_path_buf(),
            config,
        })
    }
}

pub struct Target {
    id: String,
    out_dir: PathBuf,
    file_name_template: String,

    suite_template: Option<String>,
    group_template: Option<String>,
    test_template: Option<String>,
    single_file_template: Option<String>,
}

const DEFAULT_SINGLE_FILE_TARGETS: [&str; 1] = ["markdown"];
const DEFAULT_MULTI_FILE_TARGETS: [&str; 1] = ["pytest"];

impl Target {
    pub fn from_config(config: &TargetConfig, id: &str, config_root: &Path) -> Result<Self> {
        match id {
            "pytest" => {
                return Ok(Self {
                    id: id.to_string(),
                    file_name_template: "test_{{ suite.name }}.py".to_string(),
                    out_dir: config_root.join(&config.out_dir),
                    suite_template: Some(
                        include_str!("../templates/pytest/suite.py.jinja").to_string(),
                    ),
                    group_template: Some(
                        include_str!("../templates/pytest/group.py.jinja").to_string(),
                    ),
                    test_template: Some(
                        include_str!("../templates/pytest/test.py.jinja").to_string(),
                    ),
                    single_file_template: None,
                });
            }
            "markdown" => {
                return Ok(Self {
                    id: id.to_string(),
                    file_name_template: "{{ name | convert_case('Snake') }}.md".to_string(),
                    out_dir: config_root.join(&config.out_dir),
                    suite_template: None,
                    group_template: None,
                    test_template: None,
                    single_file_template: Some(
                        include_str!("../templates/markdown/single_file.md.jinja").to_string(),
                    ),
                });
            }
            _ => {
                println!("Loading custom target: {}", id);
            }
        };

        let mut target = Self {
            id: id.to_string(),
            out_dir: config_root.join(&config.out_dir),
            file_name_template: config
                .file_name_template
                .as_ref()
                .ok_or(anyhow!(
                    "file_name_template option is missing for target {}",
                    id
                ))?
                .to_string(),
            suite_template: None,
            group_template: None,
            test_template: None,
            single_file_template: None,
        };

        let mut loaded_suite = false;
        if let Some(suite_template_path) = &config.suite_template_path {
            if DEFAULT_SINGLE_FILE_TARGETS.contains(&id) {
                return Err(anyhow!(
                    "Suite template path provided for single file target"
                ));
            }
            target.suite_template = Some(
                std::fs::read_to_string(suite_template_path)
                    .context(format!("failed to read suite template file for {}", id))?,
            );
            loaded_suite = true;
        }

        let mut loaded_group = false;
        if let Some(group_template_path) = &config.group_template_path {
            if !loaded_suite {
                return Err(anyhow!(
                    "Group template path provided without suite template path"
                ));
            }
            target.group_template = Some(
                std::fs::read_to_string(group_template_path)
                    .context(format!("failed to read group template file for {}", id))?,
            );
            loaded_group = true;
        }

        if let Some(test_template_path) = &config.test_template_path {
            if !loaded_group {
                return Err(anyhow!(
                    "Test template path provided without group template path"
                ));
            }
            target.test_template = Some(
                std::fs::read_to_string(test_template_path)
                    .context(format!("failed to read test template file for {}", id))?,
            );
        }

        if let Some(single_file_template_path) = &config.single_file_template_path {
            if DEFAULT_MULTI_FILE_TARGETS.contains(&id) {
                return Err(anyhow!(
                    "Single file template path provided for multi file target"
                ));
            }

            if loaded_suite {
                return Err(anyhow!(
                    "Cannot provide single file template path with suite template path"
                ));
            }
            target.single_file_template = Some(
                std::fs::read_to_string(single_file_template_path).context(format!(
                    "failed to read single file template file for {}",
                    id
                ))?,
            );
        }

        if target.suite_template.is_none() && target.single_file_template.is_none() {
            return Err(anyhow!("No suite or single file template provided"));
        }

        Ok(target)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub name: String,

    #[serde(rename = "target")]
    pub targets: HashMap<String, TargetConfig>,

    #[serde(rename = "suite")]
    pub suites: Vec<SuiteConfig>,

    #[serde(rename = "group")]
    pub groups: Vec<GroupConfig>,

    #[serde(rename = "test")]
    pub tests: Vec<TestConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TargetConfig {
    out_dir: PathBuf,

    file_name_template: Option<String>,
    suite_template_path: Option<PathBuf>,
    group_template_path: Option<PathBuf>,
    test_template_path: Option<PathBuf>,
    single_file_template_path: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SuiteConfig {
    pub id: String,
    pub name: Option<String>,
    pub groups: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GroupConfig {
    pub id: String,
    pub tests: Vec<String>,
    pub name: Option<String>,
    pub desc: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TestConfig {
    pub id: String,
    pub name: Option<String>,
    pub desc: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Suite {
    pub name: String,
    pub groups: Vec<Group>,
}

impl Suite {
    pub fn from_config(config: &Config, suite_config: &SuiteConfig) -> Self {
        Self {
            name: suite_config.name.clone().unwrap_or(suite_config.id.clone()),
            groups: config
                .groups
                .iter()
                .filter(|g| suite_config.groups.contains(&g.id))
                .map(|g| Group::from_config(config, g))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Group {
    pub name: String,
    pub tests: Vec<Test>,
    pub desc: String,
}

impl Group {
    pub fn from_id(config: &Config, group_id: &str) -> Self {
        let group_config = config
            .groups
            .iter()
            .find(|g| g.id == group_id)
            .expect("group should exist");
        Self::from_config(config, group_config)
    }

    fn from_config(config: &Config, group_config: &GroupConfig) -> Self {
        let tests: Vec<Test> = group_config
            .tests
            .iter()
            .map(|test_id| {
                let test = config
                    .tests
                    .iter()
                    .find(|t| &t.id == test_id)
                    .expect("test should exist");

                Test {
                    name: test.name.clone().unwrap_or(test.id.clone()),
                    desc: test.desc.clone().unwrap_or("".to_string()),
                }
            })
            .collect();

        Self {
            name: group_config.name.clone().unwrap_or(group_config.id.clone()),
            tests,
            desc: group_config.desc.clone().unwrap_or("".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Test {
    pub name: String,
    pub desc: String,
}

impl Test {
    pub fn from_config(test_config: &TestConfig) -> Self {
        Self {
            name: test_config.name.clone().unwrap_or(test_config.id.clone()),
            desc: test_config.desc.clone().unwrap_or("".to_string()),
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate test files
    Generate(Generate),
}

#[derive(Args)]
struct Generate {
    /// The target to generate tests for
    #[arg(short, long)]
    target: Option<Vec<String>>,
}

fn main() -> Result<()> {
    let parsed = Cli::parse();
    let config_meta = ConfigMeta::from_file("examples/vehicles/polytest.toml")?;

    match parsed.command {
        Commands::Generate(generate) => {
            let targets = generate
                .target
                .unwrap_or(vec!["pytest".to_string(), "markdown".to_string()]);

            for target in targets {
                generate_target(&config_meta, &target)?;
            }
        }
    }

    Ok(())
}

fn generate_target(config_meta: &ConfigMeta, target_id: &str) -> Result<()> {
    let mut env = minijinja::Environment::new();
    env.add_filter("convert_case", convert_case_filter);
    env.set_lstrip_blocks(true);
    env.set_trim_blocks(true);

    let target_config = config_meta
        .config
        .targets
        .get(target_id)
        .context(format!("could not find config for {}. Add [target.{}] to your configuration file with the required values", target_id, target_id))?;

    let target = Target::from_config(target_config, target_id, &config_meta.root_dir)?;

    if let Some(single_file_template) = &target.single_file_template {
        let template_name = format!("{}_single_file", target_id);
        env.add_template(template_name.as_str(), single_file_template)
            .context(format!(
                "failed to add template for {} single file",
                target_id
            ))?;

        let file_template_name = format!("{}_file_name", target_id);
        env.add_template(file_template_name.as_str(), &target.file_name_template)
            .context(format!(
                "failed to add template for {} file name",
                target_id
            ))?;

        generate_single_file(config_meta, &target, &env)?;

        env.remove_template(template_name.as_str());
        env.remove_template(file_template_name.as_str());
        Ok(())
    } else {
        let suite_template_name = format!("{}_suite", target_id);
        let suite_template = target
            .suite_template
            .as_ref()
            .expect("suite_template should be set by from_config");
        env.add_template(suite_template_name.as_str(), suite_template)
            .context(format!("failed to add template for {} suite", target_id))?;

        let group_template_name = format!("{}_group", target_id);
        let group_template = target
            .group_template
            .as_ref()
            .expect("group_template should be set by from_config");
        env.add_template(group_template_name.as_str(), group_template)
            .context(format!("failed to add template for {} group", target_id))?;

        let test_template_name = format!("{}_test", target_id);
        let test_template = target
            .test_template
            .as_ref()
            .expect("test_template should be set by from_config");
        env.add_template(test_template_name.as_str(), test_template)
            .context(format!("failed to add template for {} test", target_id))?;

        let file_template_name = format!("{}_file_name", target_id);
        env.add_template(file_template_name.as_str(), &target.file_name_template)
            .context(format!(
                "failed to add template for {} file name",
                target_id
            ))?;

        generate_multi_file(config_meta, &target, &env)?;

        env.remove_template(suite_template_name.as_str());
        env.remove_template(group_template_name.as_str());
        env.remove_template(test_template_name.as_str());
        env.remove_template(file_template_name.as_str());
        Ok(())
    }
}

fn generate_multi_file(
    config_meta: &ConfigMeta,
    target: &Target,
    env: &minijinja::Environment,
) -> Result<()> {
    let config = &config_meta.config;

    let suite_values: Vec<Suite> = config
        .suites
        .iter()
        .map(|s| Suite::from_config(config, s))
        .collect();

    let suite_template_name = format!("{}_suite", target.id);
    let suite_template = env
        .get_template(suite_template_name.as_str())
        .expect("suite template should have been adedd by generate_target");

    let group_template_name = format!("{}_group", target.id);
    let group_template = env
        .get_template(group_template_name.as_str())
        .expect("group template should have been adedd by generate_target");

    let test_template_name = format!("{}_test", target.id);
    let test_template = env
        .get_template(test_template_name.as_str())
        .expect("test template should have been adedd by generate_target");

    let file_template_name = format!("{}_file_name", target.id);
    let file_template = env
        .get_template(file_template_name.as_str())
        .expect("file template should have been adedd by generate_target");

    for suite in &suite_values {
        let suite_file_name = file_template
            .render(minijinja::context! {
                suite => minijinja::Value::from_serialize(suite),
            })
            .context(format!("failed to render file name for {}", target.id))?;

        let suite_file = target.out_dir.join(&suite_file_name);

        let mut contents = suite_template
            .render(minijinja::context! {
                suite => minijinja::Value::from_serialize(suite),
            })
            .context(format!("failed to render suite for {}", target.id))?;

        let suite_comment = get_suite_comment(&suite.name);

        for group in &suite.groups {
            let rendered_group = group_template
                .render(minijinja::context! {
                    group => minijinja::Value::from_serialize(group),
                })
                .context(format!(
                    "failed to render group {} for {}",
                    group.name, target.id
                ))?;

            contents = insert_after_keyword(&contents, &rendered_group, &suite_comment);

            for test in &group.tests {
                let rendered_test = test_template
                    .render(minijinja::context! {
                        test => minijinja::Value::from_serialize(test),
                        group_name => minijinja::Value::from(&group.name),
                    })
                    .context(format!(
                        "failed to render test {} for group {}",
                        test.name, group.name
                    ))?;

                let group_comment = get_group_comment(&group.name);

                contents = insert_after_keyword(&contents, &rendered_test, &group_comment);
            }
        }

        std::fs::write(&suite_file, contents).context(format!(
            "failed to write suite file {} for {}",
            &suite_file_name, target.id
        ))?;
    }

    Ok(())
}

fn generate_single_file(
    config_meta: &ConfigMeta,
    target: &Target,
    env: &minijinja::Environment,
) -> Result<()> {
    let config = &config_meta.config;
    let suite_values: Vec<Suite> = config
        .suites
        .iter()
        .map(|s| Suite::from_config(config, s))
        .collect();

    let group_values: Vec<Group> = config
        .groups
        .iter()
        .map(|g| Group::from_config(config, g))
        .collect();

    let test_values: Vec<Test> = config.tests.iter().map(Test::from_config).collect();

    let template = env
        .get_template(format!("{}_single_file", target.id).as_str())
        .expect("single file template should have been adedd by generate_target");

    let content = template
        .render(minijinja::context! {
            suites => minijinja::Value::from_serialize(&suite_values),
            groups => minijinja::Value::from_serialize(&group_values),
            tests => minijinja::Value::from_serialize(&test_values)
        })
        .context(format!("failed to render single file for {}", target.id))?;

    let file_name_template = env
        .get_template(format!("{}_file_name", target.id).as_str())
        .expect("file name template should have been adedd by generate_target");

    let file_name = file_name_template
        .render(minijinja::context! {
            name => minijinja::Value::from(&config.name),
        })
        .context(format!("failed to render file name for {}", target.id))?;

    let file_path = target.out_dir.join(file_name);
    std::fs::write(file_path, content)
        .context(format!("failed to write single file for {}", target.id))?;

    Ok(())
}
