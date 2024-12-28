use clap::{command, Args, Parser, Subcommand};
use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(rename = "suite")]
    pub suites: Vec<SuiteConfig>,

    #[serde(rename = "group")]
    pub groups: Vec<GroupConfig>,

    #[serde(rename = "test")]
    pub tests: Vec<TestConfig>,
}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let contents = std::fs::read_to_string(path).unwrap();
        toml::from_str(&contents).unwrap()
    }
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
    /// The language to generate tests for
    #[arg(short, long)]
    language: Option<Vec<String>>,
}

fn main() {
    let parsed = Cli::parse();
    let config = Config::from_file("examples/vehicles/polytest.toml");
    match parsed.command {
        Commands::Generate(generate) => {
            let languages = generate
                .language
                .unwrap_or(vec!["python".to_string(), "markdown".to_string()]);

            for language in languages {
                match language.as_str() {
                    "python" => render_pytest(&config),
                    "markdown" => render_markdown(&config),
                    _ => panic!("Unsupported language: {}", language),
                }
            }
        }
    }
}

fn render_pytest(config: &Config) {
    let mut env = minijinja::Environment::new();
    env.add_filter("convert_case", convert_case_filter);
    env.add_template("pytest", include_str!("../templates/pytest.py.jinja"))
        .unwrap();

    let suite_values: Vec<Suite> = config
        .suites
        .iter()
        .map(|s| Suite::from_config(config, s))
        .collect();

    let py_template = env.get_template("pytest").unwrap();

    for suite in suite_values {
        let pytest = py_template
            .render(minijinja::context! {
                suite => minijinja::Value::from_serialize(&suite),
            })
            .unwrap();

        std::fs::write(
            format!("examples/vehicles/py/test_{}.py", suite.name),
            pytest,
        )
        .unwrap();
    }
}

fn render_markdown(config: &Config) {
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

    let mut env = minijinja::Environment::new();
    env.add_template("markdown", include_str!("../templates/markdown.jinja"))
        .unwrap();
    env.add_filter("convert_case", convert_case_filter);

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
