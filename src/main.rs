use anyhow::{anyhow, Context, Result};
use clap::{command, Args, Parser, Subcommand};
use duct::cmd;
use duct::Handle;
use indexmap::IndexMap;
use json_comments::StripComments;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use crate::render::Renderer;
use crate::target::CustomTargetConfig;
use crate::target::DefaultTarget;
use crate::target::Target;
use crate::target::TargetConfig;
use crate::validate::validate_target;

mod parsing;
mod render;
mod runner;
mod target;
mod validate;

enum TemplateType {
    Suite,
    Group,
    Test,
}

#[derive(Clone)]
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

    let targets_clone = all_targets.clone();
    let renderer = Renderer::new(&targets_clone, config_meta.clone())?;

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
                renderer.generate_suite(&target)?;
            }

            for document in documents {
                renderer.generate_document(&document)?;
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
                validate_target(&config_meta, &target, &renderer)?;
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
                    let rendered_cmd = renderer.render_cmd(runner)?;

                    println!("Running {} > {}: {}", target.id, runner_id, rendered_cmd);

                    let parsed_cmd: Vec<String> = shlex::Shlex::new(&rendered_cmd).collect();

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

                let mut fails: Vec<String> = Vec::new();

                if !run.no_parse {
                    for (suite_id, suite_config) in &config_meta.config.suites {
                        let suite = Suite::from_config(&config_meta.config, suite_config, suite_id);
                        let suite_file_name = renderer.render_suite_file_name(target, &suite)?;
                        for group in &suite.groups {
                            for test in &group.tests {
                                if test.exclude_targets.contains(&target_id) {
                                    continue;
                                }

                                let fail_regex = renderer.render_fail_regex(
                                    &target_runner,
                                    &suite_file_name,
                                    &suite,
                                    group,
                                    test,
                                )?;

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
                                    let pass_regex = renderer.render_pass_regex(
                                        &target_runner,
                                        &suite_file_name,
                                        &suite,
                                        group,
                                        test,
                                    )?;

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
