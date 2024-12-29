use clap::{command, Args, Parser, Subcommand};
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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

fn case_from_str(s: &str) -> Result<Case, String> {
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
        _ => Err(format!(
            "Unsupported case: {}. Supported cases are: Alternating, Camel, Cobol, Flat, Kebab, \
             Lower, Pascal, Snake, ScreamingSnake/UpperSnake, Title, Toggle, Train, Upper, \
             UpperCamel, UpperFlat, UpperKebab",
            s,
        )),
    }
}

pub fn convert_case_filter(input: &str, case: &str) -> String {
    input.to_case(case_from_str(case).unwrap())
}

pub struct ConfigMeta {
    root_dir: PathBuf,
    config: Config,
}

impl ConfigMeta {
    pub fn from_file(path: &str) -> Self {
        let contents = std::fs::read_to_string(path).unwrap();
        let config = toml::from_str(&contents).unwrap();
        Self {
            root_dir: PathBuf::from(path).parent().unwrap().to_path_buf(),
            config,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
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
    pub out_dir: PathBuf,
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

fn main() {
    let parsed = Cli::parse();
    let config_meta = ConfigMeta::from_file("examples/vehicles/polytest.toml");

    match parsed.command {
        Commands::Generate(generate) => {
            let targets = generate
                .target
                .unwrap_or(vec!["pytest".to_string(), "markdown".to_string()]);

            let mut env = minijinja::Environment::new();
            env.add_filter("convert_case", convert_case_filter);
            env.set_lstrip_blocks(true);
            env.set_trim_blocks(true);

            for target in targets {
                match target.as_str() {
                    "pytest" => render_pytest(&config_meta, &mut env),
                    "markdown" => render_markdown(&config_meta, &mut env),
                    _ => panic!("Unsupported target: {}", target),
                }
            }
        }
    }
}

fn render_pytest(config_meta: &ConfigMeta, env: &mut minijinja::Environment) {
    let config = &config_meta.config;
    let target_config = config.targets.get("pytest").unwrap();
    let out_dir = config_meta.root_dir.join(&target_config.out_dir);

    env.add_template(
        "pytest_suite",
        include_str!("../templates/pytest/suite.py.jinja"),
    )
    .unwrap();

    env.add_template(
        "pytest_group",
        include_str!("../templates/pytest/group.py.jinja"),
    )
    .unwrap();

    env.add_template(
        "pytest_test",
        include_str!("../templates/pytest/test.py.jinja"),
    )
    .unwrap();

    let suite_values: Vec<Suite> = config
        .suites
        .iter()
        .map(|s| Suite::from_config(config, s))
        .collect();

    let suite_template = env.get_template("pytest_suite").unwrap();
    let group_template = env.get_template("pytest_group").unwrap();
    let test_template = env.get_template("pytest_test").unwrap();

    for suite in &suite_values {
        let suite_file = out_dir.join(format!("test_{}.py", &suite.name));

        let mut contents = suite_template
            .render(minijinja::context! {
                suite => minijinja::Value::from_serialize(suite),
            })
            .unwrap();

        let suite_comment = get_suite_comment(&suite.name);

        for group in &suite.groups {
            let rendered_group = group_template
                .render(minijinja::context! {
                    group => minijinja::Value::from_serialize(group),
                })
                .unwrap();

            contents = insert_after_keyword(&contents, &rendered_group, &suite_comment);

            for test in &group.tests {
                let rendered_test = test_template
                    .render(minijinja::context! {
                        test => minijinja::Value::from_serialize(test),
                        group_name => minijinja::Value::from(&group.name),
                    })
                    .unwrap();

                let group_comment = get_group_comment(&group.name);

                contents = insert_after_keyword(&contents, &rendered_test, &group_comment);
            }
        }

        std::fs::write(&suite_file, contents).unwrap();
    }
}

fn render_markdown(config_meta: &ConfigMeta, env: &mut minijinja::Environment) {
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

    env.add_template("markdown", include_str!("../templates/markdown.jinja"))
        .unwrap();

    let md_template = env.get_template("markdown").unwrap();
    let markdown = md_template
        .render(minijinja::context! {
            suites => minijinja::Value::from_serialize(&suite_values),
            groups => minijinja::Value::from_serialize(&group_values),
            tests => minijinja::Value::from_serialize(&test_values)
        })
        .unwrap();

    std::fs::write("examples/vehicles/generated_markdown.md", markdown).unwrap();
}
