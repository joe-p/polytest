use anyhow::{anyhow, Context, Result};
use clap::{command, Args, Parser, Subcommand};
use convert_case::{Case, Casing};
use duct::cmd;
use duct::Handle;
use glob::glob;
use indexmap::IndexMap;
use json_comments::StripComments;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use crate::generate::generate_document;
use crate::generate::generate_suite;
use crate::target::CustomTargetConfig;
use crate::target::DefaultTarget;
use crate::target::Target;
use crate::target::TargetConfig;
use crate::validate::validate_target;

mod generate;
mod runner;
mod target;
mod validate;

const GROUP_COMMENT: &str = "Polytest Group:";
const SUITE_COMMENT: &str = "Polytest Suite:";

enum TemplateType {
    Suite,
    Group,
    Test,
}

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
            "markdown" => Ok(Self {
                out_file: config_root.join(&config.out_file),
                template: config.template.clone().unwrap_or_else(|| {
                    include_str!("../templates/markdown/plan.md.jinja").to_string()
                }),
            }),
            _ => {
                let template_path = config_root.join(
                    config
                        .template
                        .clone()
                        .context(format!("{} is not a default document id and the template field is required for custom documents", id))?,
                );

                let template = std::fs::read_to_string(template_path)
                    .context(format!("failed to read template file for {}", id))?;

                Ok(Self {
                    out_file: config_root.join(&config.out_file),
                    template,
                })
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

#[derive(Deserialize, Debug, Clone)]
struct Config {
    name: String,

    package_name: String,

    #[serde(rename = "document")]
    #[serde(default)]
    documents: HashMap<String, DocumentConfig>,

    #[serde(rename = "target")]
    #[serde(default)]
    targets: HashMap<String, TargetConfig>,

    #[serde(rename = "custom_target")]
    #[serde(default)]
    custom_targets: HashMap<String, CustomTargetConfig>,

    #[serde(rename = "suite")]
    suites: IndexMap<String, SuiteConfig>,

    #[serde(rename = "group")]
    groups: IndexMap<String, GroupConfig>,
}

#[derive(Deserialize, Debug, Clone)]
struct SuiteConfig {
    groups: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct GroupConfig {
    desc: Option<String>,

    #[serde(rename = "test")]
    tests: IndexMap<String, TestConfig>,
}

#[derive(Deserialize, Debug, Clone)]
struct TestConfig {
    #[serde(default)]
    exclude_targets: Vec<String>,
    desc: Option<String>,
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
                        Some(Group::from_config(g, id))
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
    fn from_config(group_config: &GroupConfig, group_id: &str) -> Self {
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

#[derive(Debug, Serialize)]
struct Test {
    name: String,
    desc: String,
    exclude_targets: Vec<String>,
}

impl Test {
    fn from_config(test_config: &TestConfig, test_id: &str) -> Self {
        Self {
            exclude_targets: test_config.exclude_targets.clone(),
            name: test_id.to_string(),
            desc: test_config.desc.clone().unwrap_or("".to_string()),
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Path to the config file (supports .json and .toml)
    #[arg(short, long, default_value = "polytest.json")]
    config: String,

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

    /// Dump default target configurations
    DumpDefaultTargets,
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

    /// Do not run the test runners in parallel
    #[arg(long, default_value_t = false)]
    no_parallel: bool,
}

struct ActiveRunner {
    target: Target,
    runner_id: String,
    handle: Handle,
}

fn main() -> Result<()> {
    let parsed = Cli::parse();

    let config_path =
        if parsed.config == "polytest.json" && !std::path::Path::new("polytest.json").exists() {
            if std::path::Path::new("polytest.toml").exists() {
                "polytest.toml"
            } else {
                &parsed.config
            }
        } else {
            &parsed.config
        };

    let config_meta = ConfigMeta::from_file(config_path)?;

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
        .map(|(id, config)| Target::from_config(&config, &id, &config_meta.root_dir))
        .chain(
            config_meta
                .config
                .custom_targets
                .clone()
                .into_iter()
                .map(|(id, config)| {
                    Target::from_custom_config(&config, &id, &config_meta.root_dir)
                }),
        )
        .collect::<Result<Vec<Target>>>()?;

    for target in &all_targets {
        for (runner_id, runner) in &target.runners {
            let target_runner = target.id.clone() + runner_id;

            templates.insert(
                format!("{}_fail_regex", target_runner),
                runner.fail_regex_template.clone(),
            );
            templates.insert(
                format!("{}_pass_regex", target_runner),
                runner.pass_regex_template.clone(),
            );
        }

        templates.insert(
            format!("{}_suite_file_name", target.id),
            target.suite_file_name_template.to_string(),
        );

        templates.insert(
            format!("{}_test_regex", target.id),
            target.test_regex_template.to_string(),
        );

        templates.insert(
            format!("{}_suite", target.id),
            target.suite_template.to_string(),
        );

        templates.insert(
            format!("{}_group", target.id),
            target.group_template.to_string(),
        );

        templates.insert(
            format!("{}_test", target.id),
            target.test_template.to_string(),
        );
    }

    templates.iter().for_each(|(name, template)| {
        env.add_template(name, template).unwrap();
    });

    match parsed.command {
        Commands::DumpDefaultTargets => {
            let default_targets = vec![
                DefaultTarget::Pytest,
                DefaultTarget::Bun,
                DefaultTarget::Vitest,
                DefaultTarget::Swift,
            ];

            let mut custom_target_configs = IndexMap::<String, CustomTargetConfig>::new();
            for default_target in default_targets {
                let target = default_target.build_target(
                    &default_target.to_string(),
                    &TargetConfig {
                        out_dir: PathBuf::from("tests/generated"),
                        runners: None,
                    },
                    &config_meta.root_dir,
                )?;
                let custom_target_config: CustomTargetConfig = target.into();
                custom_target_configs.insert(default_target.to_string(), custom_target_config);
            }

            let serialized = serde_json::to_string_pretty(&custom_target_configs)
                .context("failed to serialize default target config")?;
            println!("{}", serialized);
        }
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
                generate_suite(&config_meta, &target, &env)?;
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
            let mut statuses = IndexMap::<(String, String), ExitStatus>::new();
            let mut outputs = HashMap::<String, String>::new();

            let targets = match run.target {
                Some(target_ids) => all_targets
                    .into_iter()
                    .filter(|target| target_ids.contains(&target.id))
                    .collect(),

                None => all_targets,
            };

            let mut active_runners = Vec::<ActiveRunner>::new();

            for target in &targets {
                for (runner_id, runner) in &target.runners {
                    let rendered_cmd = &env.render_str(
                        &runner.command,
                        minijinja::context! {
                            package_name => minijinja::Value::from(&config_meta.config.package_name),
                        },
                    )?;

                    println!("Running {} > {}: {}", target.id, runner_id, rendered_cmd);

                    let parsed_cmd: Vec<String> = shlex::Shlex::new(rendered_cmd).collect();

                    let mut runner_cmd = cmd(&parsed_cmd[0], &parsed_cmd[1..])
                        .dir(config_meta.root_dir.join(runner.work_dir.clone()))
                        .unchecked()
                        .stderr_to_stdout();

                    if let Some(env) = &runner.env {
                        for (key, value) in env {
                            runner_cmd = runner_cmd.env(key, value);
                        }
                    }

                    if run.no_parallel {
                        let reader = runner_cmd.reader()?;
                        let output = &mut String::new();
                        BufReader::new(reader).read_to_string(output)?;

                        statuses.insert(
                            (target.id.clone(), runner_id.clone()),
                            runner_cmd.run()?.status,
                        );

                        outputs.insert(target.id.clone() + runner_id, output.clone());
                    } else {
                        active_runners.push(ActiveRunner {
                            handle: runner_cmd.stdout_capture().start()?,
                            target: target.clone(),
                            runner_id: runner_id.to_string(),
                        })
                    }
                }
            }

            while !active_runners.is_empty() {
                let mut idx_for_removal = Vec::<usize>::new();
                for (i, runner) in active_runners.iter().enumerate() {
                    if let Some(status) = runner.handle.try_wait()? {
                        idx_for_removal.push(i);

                        let output = String::from_utf8(status.stdout.clone())?;
                        println!("{output}");
                        let target_id = runner.target.id.clone();
                        outputs.insert(target_id.clone() + &runner.runner_id, output);

                        statuses.insert((target_id, runner.runner_id.clone()), status.status);
                    }
                }

                for i in idx_for_removal.iter().rev() {
                    active_runners.remove(*i);
                }

                // Add a small sleep to prevent busy-waiting
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            for ((target_id, runner_id), status) in statuses {
                let target_runner = target_id.clone() + &runner_id;

                let target = targets
                    .iter()
                    .find(|t| t.id == target_id)
                    .expect("the target should exist because the status exists");

                let fail_regex_template = env
                    .get_template(format!("{}_fail_regex", target_runner).as_str())
                    .expect("template should exist since it was just added above");

                let pass_regex_template = env
                    .get_template(format!("{}_pass_regex", target_runner).as_str())
                    .expect("template should exist since it was just added above");

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
                                if test.exclude_targets.contains(&target_id) {
                                    continue;
                                }

                                let fail_regex = fail_regex_template
                                    .render(minijinja::context! {
                                        file_name => minijinja::Value::from(&suite_file_name),
                                        suite_name => minijinja::Value::from(&suite.name),
                                        group_name => minijinja::Value::from(&group.name),
                                        test_name => minijinja::Value::from(&test.name),
                                    })
                                    .context(format!(
                                        "failed to render fail regex for {}",
                                        target_runner
                                    ))?;

                                let fail_regex = Regex::new(&fail_regex).unwrap();

                                if fail_regex.is_match(&outputs[&target_runner]) {
                                    fails.push(
                                        format!(
                                            "  {} ({}) > {} > {} > {}: FAILED",
                                            target_id, runner_id, suite.name, group.name, test.name
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
                                            target_runner
                                        ))?;

                                    let pass_regex = Regex::new(&pass_regex).unwrap();

                                    if !pass_regex.is_match(&outputs[&target_runner]) {
                                        fails.push(
                                            format!(
                                                "  {} ({}) > {} > {} > {}: UNKNOWN: could not find either regex:\n    FAIL REGEX: {}\n    PASS REGEX: {}",
                                                target_id,
                                                runner_id,
                                                suite.name,
                                                group.name,
                                                test.name,
                                                fail_regex.as_str(),
                                                pass_regex.as_str()
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
                    println!("{} ({}): ran succesfully!", target_id, runner_id);
                } else {
                    eprintln!(
                        "{} ({}): failed to run succesfully ({})",
                        target_id, runner_id, status
                    );
                    for failure in &fails {
                        eprintln!("{failure}")
                    }
                }
            }
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

pub struct SuiteChunk {
    content: String,
    start: usize,
    end: usize,
}

/// Gets the chunk of the input that starts with the suite comment and ends with
/// the next suite comment (or the end of the file)
pub fn get_suite_chunk(input: &str, name: &str) -> Result<SuiteChunk> {
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

pub fn find_test(
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
