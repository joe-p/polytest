use anyhow::{anyhow, Context, Result};
use clap::{command, Args, Parser, Subcommand};
use convert_case::{Case, Casing};
use duct::cmd;
use glob::glob;
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

const GROUP_COMMENT: &str = "Polytest Group:";
const SUITE_COMMENT: &str = "Polytest Suite:";

fn get_group_comment(group: &str) -> String {
    format!("{} {}", GROUP_COMMENT, group)
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

fn convert_case_filter(input: &str, case: &str) -> String {
    input.to_case(case_from_str(case).unwrap_or_else(|e| {
        panic!("failed to convert case: {}", e);
    }))
}

struct ConfigMeta {
    root_dir: PathBuf,
    config: Config,
}

impl ConfigMeta {
    fn from_file(path: &str) -> Result<Self> {
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

#[derive(Deserialize, Debug, Clone)]
struct RunnerConfig {
    command: String,
    args: Vec<String>,
    fail_regex_template: String,
    pass_regex_template: String,
    env: Option<HashMap<String, String>>,
    work_dir: Option<PathBuf>,
}

#[derive(Deserialize, Debug, Clone)]
struct Runner {
    command: String,
    args: Vec<String>,
    fail_regex_template: String,
    pass_regex_template: String,
    env: Option<HashMap<String, String>>,
    work_dir: Option<PathBuf>,
}

impl Runner {
    fn from_config(config: &RunnerConfig) -> Self {
        Self {
            command: config.command.clone(),
            args: config.args.clone(),
            fail_regex_template: "(?m)".to_owned() + config.fail_regex_template.as_str(),
            pass_regex_template: "(?m)".to_owned() + config.pass_regex_template.as_str(),
            env: config.env.clone(),
            work_dir: config.work_dir.clone(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Target {
    id: String,
    out_dir: PathBuf,
    test_regex_template: Option<String>,
    suite_file_name_template: Option<String>,
    suite_template: Option<String>,
    group_template: Option<String>,
    test_template: Option<String>,
    runner: Option<Runner>,
}

#[derive(Deserialize, Debug, Clone)]
struct DocumentConfig {
    out_file: PathBuf,
    template: Option<String>,
}

struct Document {
    out_file: PathBuf,
    template: String,
}

impl Document {
    fn from_config(config: &DocumentConfig, id: &str, config_root: &Path) -> Result<Self> {
        match id {
            "markdown" => {
                return Ok(Self {
                    out_file: config.out_file.clone(),
                    template: config.template.clone().unwrap_or_else(|| {
                        include_str!("../templates/markdown/plan.md.jinja").to_string()
                    }),
                });
            }
            _ => {
                let template_path = config_root.join(
                    config
                        .template
                        .clone()
                        .context("template is required for custom documents")?,
                );

                let template = std::fs::read_to_string(template_path)
                    .context(format!("failed to read template file for {}", id))?;

                return Ok(Self {
                    out_file: config.out_file.clone(),
                    template,
                });
            }
        }
    }
}

fn find_template_file(template_dir: &Path, template_name: &str) -> Result<PathBuf> {
    let pattern = template_dir.join(template_name).to_path_buf();
    glob(pattern.to_str().unwrap())?
        .next()
        .ok_or_else(|| anyhow!("No template file found matching: {} ", pattern.display()))
        .and_then(|path| path.map_err(|e| e.into()))
}

impl Target {
    fn from_config(
        config: &TargetConfig,
        id: &str,
        config_root: &Path,
        is_custom: bool,
    ) -> Result<Self> {
        if !is_custom {
            match id {
                "pytest" => {
                    return Ok(Self {
                        id: id.to_string(),
                        test_regex_template: Some(
                            r"(?m)def test_{{ name | convert_case('Snake') }}\(".to_string(),
                        ),
                        suite_file_name_template: Some(
                            "test_{{ suite.name | convert_case('Snake') }}.py".to_string(),
                        ),
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
                        runner: config
                            .runner
                            .as_ref()
                            .map(|runner_config| Runner::from_config(&runner_config))
                            .or(Some(Runner {
                                env: None,
                                command: "pytest".to_string(),
                                args: vec!["-v".to_string()],
                                fail_regex_template:
                                    r"(?m){{ file_name }}::test_{{ test_name }} FAILED".to_string(),
                                pass_regex_template:
                                    r"(?m){{ file_name }}::test_{{ test_name }} PASSED".to_string(),
                                work_dir: None,
                            })),
                    });
                }
                "bun" => {
                    return Ok(Self {
                    id: id.to_string(),
                    test_regex_template: Some(r#"(?m)test\("{{ name }}","#.to_string()),
                    suite_file_name_template: Some(
                        "{{ suite.name | convert_case('Snake') }}.test.ts".to_string(),
                    ),
                    out_dir: config_root.join(&config.out_dir),
                    suite_template: Some(
                        include_str!("../templates/bun/suite.ts.jinja").to_string(),
                    ),
                    group_template: Some(
                        include_str!("../templates/bun/group.ts.jinja").to_string(),
                    ),
                    test_template: Some(include_str!("../templates/bun/test.ts.jinja").to_string()),
                    runner: config
                        .runner
                        .as_ref()
                        .map(|runner_config| Runner::from_config(&runner_config))
                        .or(Some(Runner {
                            env: None,
                            command: "bun".to_string(),
                            args: vec!["test".to_string()],
                            fail_regex_template: r"(?m)\(fail\) {{ suite_name }} > {{ group_name }} > {{ test_name }}( \[\d+\.\d+ms])*$".to_string(),
                            pass_regex_template: r"(?m)\(pass\) {{ suite_name }} > {{ group_name }} > {{ test_name }}( \[\d+\.\d+ms])*$".to_string(),
                            work_dir: None,
                        })),
                });
                }
                _ => {
                    anyhow!("config defined for target {} but this is not a supported target. Perhaps you meant to use custom_target?", id)
                }
            };
        }

        let mut target = Self {
            id: id.to_string(),
            test_regex_template: config
                .test_regex_template
                .clone()
                .map(|t| "(?m)".to_owned() + t.as_str()),
            out_dir: config_root.join(&config.out_dir),
            suite_file_name_template: config.suite_file_name_template.clone(),
            suite_template: None,
            group_template: None,
            test_template: None,
            runner: config
                .runner
                .as_ref()
                .map(|runner_config| Runner::from_config(&runner_config)),
        };

        let template_dir = &config_root.join(
            config
                .template_dir
                .as_ref()
                .ok_or(anyhow!("Template directory is required for custom targets"))?,
        );

        if let Ok(suite_file) = find_template_file(template_dir, "suite*") {
            target.suite_template = Some(
                std::fs::read_to_string(suite_file)
                    .context(format!("failed to read suite template file for {}", id,))?,
            );

            let group_file = find_template_file(template_dir, "group*")?;
            target.group_template = Some(
                std::fs::read_to_string(group_file)
                    .context(format!("failed to read group template file for {}", id))?,
            );

            let test_file = find_template_file(template_dir, "test*")?;

            target.test_template = Some(
                std::fs::read_to_string(test_file)
                    .context(format!("failed to read test template file for {}", id))?,
            );
        }

        if target.suite_template.is_none() {
            return Err(anyhow!("No suite or plan template provided"));
        }

        Ok(target)
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Config {
    name: String,

    #[serde(rename = "document")]
    documents: HashMap<String, DocumentConfig>,

    #[serde(rename = "target")]
    targets: HashMap<String, TargetConfig>,

    #[serde(rename = "custom_target")]
    custom_targets: HashMap<String, CustomTargetConfig>,

    #[serde(rename = "suite")]
    suites: IndexMap<String, SuiteConfig>,

    #[serde(rename = "group")]
    groups: IndexMap<String, GroupConfig>,

    #[serde(rename = "test")]
    tests: IndexMap<String, TestConfig>,
}

#[derive(Deserialize, Debug, Clone)]
struct TargetConfig {
    out_dir: PathBuf,

    test_regex_template: Option<String>,
    suite_file_name_template: Option<String>,
    template_dir: Option<PathBuf>,
    runner: Option<RunnerConfig>,
}

#[derive(Deserialize, Debug, Clone)]
struct CustomTargetConfig {
    out_dir: PathBuf,

    test_regex_template: String,
    suite_file_name_template: String,
    template_dir: PathBuf,
    runner: RunnerConfig,
}

impl From<CustomTargetConfig> for TargetConfig {
    fn from(config: CustomTargetConfig) -> Self {
        Self {
            out_dir: config.out_dir,
            test_regex_template: Some(config.test_regex_template),
            suite_file_name_template: Some(config.suite_file_name_template),
            template_dir: Some(config.template_dir),
            runner: Some(config.runner),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct SuiteConfig {
    groups: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct GroupConfig {
    desc: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct TestConfig {
    desc: Option<String>,
    group: String,
}

#[derive(Debug, Serialize)]
struct Suite {
    name: String,
    groups: Vec<Group>,
}

impl Suite {
    fn from_config(config: &Config, suite_config: &SuiteConfig, suite_id: &str) -> Self {
        Self {
            name: suite_id.to_string(),
            groups: config
                .groups
                .iter()
                .filter_map(|(id, g)| {
                    if suite_config.groups.contains(id) {
                        Some(Group::from_config(config, g, id))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Group {
    name: String,
    tests: Vec<Test>,
    desc: String,
}

impl Group {
    fn from_config(config: &Config, group_config: &GroupConfig, group_id: &str) -> Self {
        let tests: Vec<Test> = config
            .tests
            .iter()
            .filter(|(_, test)| test.group == group_id)
            .map(|(id, test)| Test {
                name: id.to_string(),
                desc: test.desc.clone().unwrap_or("".to_string()),
            })
            .collect();

        Self {
            name: group_id.to_string(),
            tests,
            desc: group_config.desc.clone().unwrap_or("".to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
struct Test {
    name: String,
    desc: String,
}

impl Test {
    fn from_config(test_config: &TestConfig, test_id: &str) -> Self {
        Self {
            name: test_id.to_string(),
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

    /// Validate test files
    Validate(Validate),

    /// Run tests
    Run(Run),
}

#[derive(Args)]
struct Generate {
    /// A target to generate tests for
    #[arg(short, long)]
    target: Option<Vec<String>>,

    /// A document to generate
    #[arg(short, long)]
    document: Option<Vec<String>>,
}

#[derive(Args)]
struct Validate {
    /// A target to validate tests for
    #[arg(short, long)]
    target: Option<Vec<String>>,
}

#[derive(Args)]
struct Run {
    /// A target to execute the runner for
    #[arg(short, long)]
    target: Option<Vec<String>>,

    /// Do not parse the results to determine if all test cases were ran and instead just rely on
    /// exit status of the runner(s)
    #[arg(long)]
    no_parse: bool,
}

fn main() -> Result<()> {
    let parsed = Cli::parse();
    let config_meta = ConfigMeta::from_file("polytest.toml")?;

    let mut env = minijinja::Environment::new();
    env.add_filter("convert_case", convert_case_filter);
    env.set_lstrip_blocks(true);

    env.set_trim_blocks(true);

    let mut templates: HashMap<String, String> = HashMap::new();

    for (document_id, document_config) in &config_meta.config.documents {
        let document = Document::from_config(document_config, document_id, &config_meta.root_dir)?;
        templates.insert(format!("{}_document", document_id), document.template);
    }

    for target_id in config_meta.config.targets.keys() {
        if config_meta.config.custom_targets.contains_key(target_id) {
            return Err(anyhow!("{} is defined as both a target and custom_target, please change the name of the custom_target", target_id));
        }
    }

    let all_targets = config_meta
        .config
        .targets
        .clone()
        .into_iter()
        .map(|(id, config)| Target::from_config(&config, &id, &config_meta.root_dir, false))
        .chain(
            config_meta
                .config
                .custom_targets
                .clone()
                .into_iter()
                .map(|(id, config)| {
                    Target::from_config(&config.into(), &id, &config_meta.root_dir, true)
                }),
        )
        .collect::<Result<Vec<Target>>>()?;

    for target in &all_targets {
        if let Some(runner) = &target.runner {
            templates.insert(
                format!("{}_fail_regex", target.id),
                runner.fail_regex_template.clone(),
            );
            templates.insert(
                format!("{}_pass_regex", target.id),
                runner.pass_regex_template.clone(),
            );
        }

        if let Some(suite_file_name_template) = &target.suite_file_name_template {
            templates.insert(
                format!("{}_suite_file_name", target.id),
                suite_file_name_template.to_string(),
            );
        }

        if let Some(test_regex_template) = &target.test_regex_template {
            templates.insert(
                format!("{}_test_regex", target.id),
                test_regex_template.to_string(),
            );
        }

        if let Some(suite_template) = &target.suite_template {
            templates.insert(format!("{}_suite", target.id), suite_template.to_string());
        }

        if let Some(group_template) = &target.group_template {
            templates.insert(format!("{}_group", target.id), group_template.to_string());
        }

        if let Some(test_template) = &target.test_template {
            templates.insert(format!("{}_test", target.id), test_template.to_string());
        }
    }

    templates.iter().for_each(|(name, template)| {
        env.add_template(name, template).unwrap();
    });

    match parsed.command {
        Commands::Generate(generate) => {
            let targets = match generate.target {
                Some(target_ids) => all_targets
                    .into_iter()
                    .filter(|target| target_ids.contains(&target.id))
                    .collect(),

                None => all_targets,
            };

            let documents = generate
                .document
                .unwrap_or(config_meta.config.documents.keys().cloned().collect());

            for target in targets {
                generate_target(&config_meta, &target, &env)?;
            }

            for document in documents {
                generate_document(&config_meta, &document, &env)?;
            }
        }
        Commands::Validate(validate) => {
            let targets = match validate.target {
                Some(target_ids) => all_targets
                    .into_iter()
                    .filter(|target| target_ids.contains(&target.id))
                    .collect(),

                None => all_targets,
            };

            for target in targets {
                validate_target(&config_meta, &target, &env)?;
            }
        }
        Commands::Run(run) => {
            let mut statuses = IndexMap::<String, ExitStatus>::new();
            let mut outputs = HashMap::<String, String>::new();

            let targets = match run.target {
                Some(target_ids) => all_targets
                    .into_iter()
                    .filter(|target| target_ids.contains(&target.id))
                    .collect(),

                None => all_targets,
            };

            for target in &targets {
                let runner = target.runner.clone().context(format!(
                    "Attempted to execute runner for {}, but a runner is not configured",
                    target.id
                ))?;

                println!(
                    "Running {}: {} {:?}",
                    target.id, runner.command, runner.args
                );

                let mut runner_cmd = cmd(runner.command, &runner.args[..]);

                if let Some(work_dir) = &runner.work_dir {
                    runner_cmd = runner_cmd.dir(config_meta.root_dir.join(work_dir));
                }

                if let Some(env) = runner.env {
                    for (key, value) in env {
                        runner_cmd = runner_cmd.env(key, value);
                    }
                }

                let runner_result = runner_cmd.unchecked();
                let reader = runner_result.stderr_to_stdout().reader()?;
                let output = &mut String::new();
                BufReader::new(reader).read_to_string(output)?;

                outputs.insert(target.id.clone(), output.clone());
                statuses.insert(target.id.clone(), runner_result.run()?.status);
            }

            for (target_id, status) in statuses {
                let target = targets
                    .iter()
                    .find(|t| t.id == target_id)
                    .expect("the target should exist because the status exists");

                let fail_regex_template = env
                    .get_template(format!("{}_fail_regex", target_id).as_str())
                    .expect("template should exist since it was just added above");

                let pass_regex_template = env
                    .get_template(format!("{}_pass_regex", target_id).as_str())
                    .context(format!("could not find test pass regex for {}", target_id))?;

                let mut fails: Vec<String> = Vec::new();

                if !run.no_parse {
                    for (suite_id, suite_config) in &config_meta.config.suites {
                        let suite = Suite::from_config(&config_meta.config, suite_config, suite_id);
                        let suite_file_name = env
                            .get_template(format!("{}_suite_file_name", target_id).as_str())
                            .expect("template should exist since it was just added above")
                            .render(minijinja::context! {
                                suite => minijinja::Value::from_serialize(&suite),
                            })
                            .context(format!("failed to render file name for {}", target.id))?;

                        for group in &suite.groups {
                            for test in &group.tests {
                                let fail_regex = fail_regex_template
                                    .render(minijinja::context! {
                                        file_name => minijinja::Value::from(&suite_file_name),
                                        suite_name => minijinja::Value::from(&suite.name),
                                        group_name => minijinja::Value::from(&group.name),
                                        test_name => minijinja::Value::from(&test.name),
                                    })
                                    .context(format!(
                                        "failed to render fail regex for {}",
                                        target_id
                                    ))?;

                                let fail_regex = Regex::new(&fail_regex).unwrap();

                                if fail_regex.is_match(&outputs[&target_id]) {
                                    fails.push(
                                        format!(
                                            "  {} > {} > {} > {}: FAILED",
                                            target_id, suite.name, group.name, test.name
                                        )
                                        .to_string(),
                                    );
                                } else {
                                    let pass_regex = pass_regex_template
                                        .render(minijinja::context! {
                                            file_name => minijinja::Value::from(&suite_file_name),
                                            suite_name => minijinja::Value::from(&suite.name),
                                            group_name => minijinja::Value::from(&group.name),
                                            test_name => minijinja::Value::from(&test.name),
                                        })
                                        .context(format!(
                                            "failed to render pass regex for {}",
                                            target_id
                                        ))?;

                                    let pass_regex = Regex::new(&pass_regex).unwrap();

                                    if !pass_regex.is_match(&outputs[&target_id]) {
                                        fails.push(
                                            format!(
                                                "  {} > {} > {} > {}: UNKNOWN",
                                                target_id, suite.name, group.name, test.name
                                            )
                                            .to_string(),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                // If the command was exit 0 AND there were no FAILED or UNKNOWN test results
                if status.success() && fails.is_empty() {
                    println!("{} ran succesfully!", target_id);
                } else {
                    eprintln!("{} failed to run succesfully ({})", target_id, status);
                    for failure in &fails {
                        eprintln!("{failure}")
                    }
                }
            }
        }
    }

    Ok(())
}

fn validate_target(
    config_meta: &ConfigMeta,
    target: &Target,
    env: &minijinja::Environment,
) -> Result<()> {
    let file_template_name = format!("{}_suite_file_name", target.id);

    let suites: Vec<Suite> = config_meta
        .config
        .suites
        .iter()
        .map(|(id, s)| Suite::from_config(&config_meta.config, s, id))
        .collect();

    let test_regex_template_name = format!("{}_test_regex", target.id);

    let test_regex_template = env
        .get_template(&test_regex_template_name)
        .expect("template should exist since it was just added above");

    let file_template = env
        .get_template(file_template_name.as_str())
        .context(format!("failed to get file template for {}", target.id))?;

    for suite in &suites {
        let suite_file_name = file_template
            .render(minijinja::context! {
                suite => minijinja::Value::from_serialize(suite),
            })
            .context(format!("failed to render file name for {}", target.id))?;

        let suite_file = target.out_dir.join(&suite_file_name);
        if !suite_file.exists() {
            return Err(anyhow!(
                "suite file {} does not exist",
                suite_file.display()
            ));
        }

        let contents = std::fs::read_to_string(&suite_file).context(format!(
            "failed to read existing suite file for {}",
            target.id
        ))?;

        let suite_chunk = get_suite_chunk(&contents, &suite.name)?;
        let all_tests_regex = test_regex_template
            .render(minijinja::context! {
                name => minijinja::Value::from(".*"),
            })
            .context(format!("failed to render test regex for {}", target.id))?;

        let all_tests_regex = Regex::new(&all_tests_regex).unwrap();

        let mut remaining_tests: Vec<String> = all_tests_regex
            .find_iter(&suite_chunk.content)
            .map(|m| m.as_str().to_string())
            .collect();

        for group in &suite.groups {
            for test in &group.tests {
                if !find_test(&contents, &target, &test.name, &env)? {
                    return Err(anyhow!(
                        "test \"{}\" does not exist in {}",
                        test.name,
                        suite_file.display()
                    ));
                } else {
                    let test_regex = test_regex_template
                        .render(minijinja::context! {
                            name => minijinja::Value::from(&test.name),
                        })
                        .context(format!("failed to render test regex for {}", target.id))?;

                    let test_regex = Regex::new(&test_regex).unwrap();

                    remaining_tests = remaining_tests
                        .iter()
                        .filter(|t| !test_regex.is_match(t))
                        .cloned()
                        .collect();

                    println!("test \"{}\" exists in {}", test.name, suite_file.display());
                }
            }
        }

        if !remaining_tests.is_empty() {
            return Err(anyhow!(
                "found test implementation(s) in \"{}\" suite in {} that were not defined in the test plan\n{}",
                suite.name,
                suite_file.display(),
                remaining_tests.join("\n")
            ));
        }
    }

    Ok(())
}

fn get_groups(input: &str) -> Vec<String> {
    let re = Regex::new(format!(r"{} (.*)", GROUP_COMMENT).as_str()).unwrap();
    let mut groups = Vec::new();
    for cap in re.captures_iter(input) {
        groups.push(cap[1].to_string().trim().to_string());
    }
    groups
}

fn find_suite(input: &str, name: &str) -> Result<bool> {
    let re = Regex::new(format!(r"{} {}", SUITE_COMMENT, name).as_str()).unwrap();
    Ok(re.is_match(input))
}

struct SuiteChunk {
    content: String,
    start: usize,
    end: usize,
}

/// Gets the chunk of the input that starts with the suite comment and ends with
/// the next suite comment (or the end of the file)
fn get_suite_chunk(input: &str, name: &str) -> Result<SuiteChunk> {
    let start_re = Regex::new(format!(r"{} {}", SUITE_COMMENT, name).as_str()).unwrap();
    let start = start_re.find(input).unwrap().end();

    let end_chunk = input[start..].to_string();

    let end_re = Regex::new(SUITE_COMMENT).unwrap();
    let end = start
        + end_re
            .find(&end_chunk)
            .map(|m| m.start())
            .unwrap_or(end_chunk.len());

    Ok(SuiteChunk {
        content: input[start..end].to_string(),
        start,
        end,
    })
}

fn find_test(
    input: &str,
    target: &Target,
    name: &str,
    env: &minijinja::Environment,
) -> Result<bool> {
    let template = env.get_template(format!("{}_test_regex", target.id).as_str())?;
    let regex = template
        .render(minijinja::context! {
            name => minijinja::Value::from(name),
        })
        .context(format!("failed to render test regex for {}", target.id))?;

    let re = Regex::new(&regex).unwrap();
    Ok(re.is_match(input))
}

fn generate_target(
    config_meta: &ConfigMeta,
    target: &Target,
    env: &minijinja::Environment,
) -> Result<()> {
    generate_suite(config_meta, &target, &env)?;

    Ok(())
}

fn generate_suite(
    config_meta: &ConfigMeta,
    target: &Target,
    env: &minijinja::Environment,
) -> Result<()> {
    let config = &config_meta.config;

    let suite_values: Vec<Suite> = config
        .suites
        .iter()
        .map(|(id, s)| Suite::from_config(config, s, id))
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

    let file_template_name = format!("{}_suite_file_name", target.id);
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

        let mut contents: String = String::new();

        if suite_file.exists() {
            println!("{} exists, reading content...", suite_file.display());
            contents = std::fs::read_to_string(&suite_file).context(format!(
                "failed to read existing suite file for {}",
                target.id
            ))?;
        }

        if !find_suite(&contents, &suite.name)? {
            contents.insert_str(
                contents.len(),
                &suite_template
                    .render(minijinja::context! {
                        suite => minijinja::Value::from_serialize(suite),
                    })
                    .context(format!("failed to render suite for {}", target.id))?,
            );
        }

        let mut suite_chunk = get_suite_chunk(&contents, &suite.name)?;

        let existing_groups = get_groups(&suite_chunk.content);

        let missing_groups: Vec<&Group> = suite
            .groups
            .iter()
            .filter(|g| !existing_groups.contains(&g.name))
            .collect();

        for group in &missing_groups {
            let rendered_group = group_template
                .render(minijinja::context! {
                    group => minijinja::Value::from_serialize(group),
                })
                .context(format!(
                    "failed to render group {} for {}",
                    group.name, target.id
                ))?;

            suite_chunk.content.insert_str(0, &rendered_group);
        }

        for group in &suite.groups {
            for test in &group.tests {
                // We don't need to get a group-specific chunk because two groups can't have the same test
                if find_test(&suite_chunk.content, target, &test.name, env)? {
                    println!(
                        "test \"{}\" already exists in {}. Skipping...",
                        test.name,
                        suite_file.display()
                    );
                    continue;
                }

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

                suite_chunk.content =
                    insert_after_keyword(&suite_chunk.content, &rendered_test, &group_comment);
            }
        }

        contents.replace_range(suite_chunk.start..suite_chunk.end, &suite_chunk.content);

        if let Some(parent) = suite_file.parent() {
            std::fs::create_dir_all(parent).context(format!(
                "failed to create directory for {}",
                parent.display()
            ))?;
        }

        std::fs::write(&suite_file, contents).context(format!(
            "failed to write suite file {} for {}",
            &suite_file.display(),
            target.id
        ))?;
    }

    Ok(())
}

fn generate_document(
    config_meta: &ConfigMeta,
    doc_id: &str,
    env: &minijinja::Environment,
) -> Result<()> {
    let document_config = config_meta
        .config
        .documents
        .get(doc_id)
        .context(format!("document {} does not exist", doc_id))?;

    let document = Document::from_config(document_config, doc_id, &config_meta.root_dir)?;

    let config = &config_meta.config;
    let suite_values: Vec<Suite> = config
        .suites
        .iter()
        .map(|(id, s)| Suite::from_config(config, s, id))
        .collect();

    let group_values: Vec<Group> = config
        .groups
        .iter()
        .map(|(id, g)| Group::from_config(config, g, id))
        .collect();

    let test_values: Vec<Test> = config
        .tests
        .iter()
        .map(|(id, t)| Test::from_config(t, id))
        .collect();

    let template = env
        .template_from_str(&document.template)
        .context(format!("failed to load template for {}", doc_id))?;

    let content = template
        .render(minijinja::context! {
            name => minijinja::Value::from(&config.name),
            suites => minijinja::Value::from_serialize(&suite_values),
            groups => minijinja::Value::from_serialize(&group_values),
            tests => minijinja::Value::from_serialize(&test_values)
        })
        .context(format!("failed to render document for {}", doc_id))?;

    if let Some(parent) = document.out_file.parent() {
        std::fs::create_dir_all(parent).context(format!(
            "failed to create directory for {}",
            parent.display()
        ))?;
    }

    std::fs::write(document.out_file, content)
        .context(format!("failed to write document for {}", doc_id))?;

    Ok(())
}
