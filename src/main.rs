use clap::{command, Args, Parser, Subcommand};
use color_eyre::eyre::ensure;
use color_eyre::eyre::{eyre, Context, Result};
use duct::cmd;
use duct::Handle;
use indexmap::IndexMap;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::ExitStatus;

use crate::config::ConfigMeta;
use crate::render::Renderer;
use crate::suite::Suite;
use crate::target::CustomTargetConfig;
use crate::target::DefaultTarget;
use crate::target::Target;
use crate::target::TargetConfig;
use crate::validate::validate_target;

mod config;
mod document;
mod group;
mod parsing;
mod render;
mod runner;
mod suite;
mod target;
mod test;
mod validate;

enum TemplateType {
    Suite,
    Group,
    Test,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Path to the config file (supports .json and .jsonc). If the --git option is used, this path
    /// will be relative to the root of the cloned git repo.
    #[arg(short, long, default_value = "polytest.json")]
    config: String,

    /// The git repo to use as the working directory for polytest. If specified, the repo will be
    /// cloned to a directory in the actual working director
    #[arg(long)]
    git: Option<String>,

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

#[derive(Clone, PartialEq)]
struct GitRemote {
    /// The git remote host. For example, "github.com"
    host: String,
    /// The git repo org
    org: String,
    /// The git repo name
    repo: String,
    /// The raw git uri given, minus the branch/ref
    raw_uri: String,
}

impl GitRemote {
    fn from_url(url: &str) -> Result<Self> {
        let normalized_url = url
            .to_string()
            .replace("git+", "")
            .replace("http://", "")
            .replace("https://", "")
            .replace("ssh://", "")
            .replace("git@", "")
            .replace("git://", "")
            .trim_end_matches(".git")
            .to_string();

        let parts: Vec<&str> = normalized_url.split('/').collect();

        if parts.len() < 3 {
            return Err(eyre!("invalid git url: {}", url));
        }

        Ok(GitRemote {
            host: parts[0].to_string(),
            org: parts[1].to_string(),
            repo: parts[2].to_string(),
            raw_uri: url
                .replace("git+", "")
                .split_once('#')
                .unwrap_or((url, ""))
                .0
                .to_string(),
        })
    }
}

impl Display for GitRemote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", self.host, self.org, self.repo)
    }
}

#[derive(Clone)]
struct GitRemoteRef {
    remote: GitRemote,
    git_ref: String,
}

impl GitRemoteRef {
    fn from_url(url: &str) -> Result<Self> {
        let (url_part, ref_part) = if let Some((url_part, ref_part)) = url.split_once("#") {
            (url_part.to_string(), ref_part.to_string())
        } else {
            (url.to_string(), "main".to_string())
        };

        Ok(GitRemoteRef {
            remote: GitRemote::from_url(&url_part)?,
            git_ref: ref_part,
        })
    }
}

impl Display for GitRemoteRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}#{}", self.remote, self.git_ref)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let parsed = Cli::parse();

    if let Some(parsed_git) = &parsed.git {
        let remote_ref = GitRemoteRef::from_url(parsed_git)
            .context("failed to parse git remote and ref from url")?;
        let git_ref = remote_ref.git_ref.clone();

        let repo_dir_str = ".polytest_".to_owned() + &remote_ref.remote.repo;
        let repo_dir = std::path::Path::new(&repo_dir_str);

        if repo_dir.exists() {
            println!(
                "Using existing git repo {}. Fetching and checking out {}",
                repo_dir.display(),
                remote_ref
            );

            let repo =
                git2::Repository::open(repo_dir).context("failed to open existing git repo")?;

            let mut remote = repo.find_remote("origin")?;

            let existing_remote = GitRemote::from_url(remote.url().unwrap_or(""))
                .context("failed to parse existing git remote url")?;

            ensure!(
                existing_remote == remote_ref.remote,
                "existing git repo remote ({}) does not match requested remote ({})",
                existing_remote,
                remote_ref.remote
            );

            remote.fetch(std::slice::from_ref(&git_ref), None, None)?;

            // Get the fetch head reference
            let fetch_head =
                repo.find_reference(&format!("refs/remotes/{}/{}", "origin", git_ref))?;
            let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;

            // Create a local branch if it doesn't exist
            if repo.find_branch(&git_ref, git2::BranchType::Local).is_err() {
                repo.branch(&git_ref, &repo.find_commit(fetch_commit.id())?, false)?;
            }

            // Set HEAD to the branch
            let obj = repo.revparse_single(&format!("refs/heads/{}", git_ref))?;
            repo.checkout_tree(&obj, None)?;
            repo.set_head(&format!("refs/heads/{}", git_ref))?;
        } else {
            println!("Cloning {} into {}", remote_ref, repo_dir.display());

            git2::build::RepoBuilder::new()
                .branch(&git_ref)
                .clone(&remote_ref.remote.raw_uri, std::path::Path::new(repo_dir))
                .context("failed to clone git repo")?;
        }

        env::set_current_dir(std::path::Path::new(repo_dir))
            .context("failed to change working directory to git repo")?;
    }

    let config_meta = ConfigMeta::from_file(&parsed.config)?;

    for target_id in config_meta.config.targets.keys() {
        if config_meta.config.custom_targets.contains_key(target_id) {
            return Err(eyre!("{} is defined as both a target and custom_target, please change the name of the custom_target", target_id));
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
                        resource_dir: None,
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

            // Copy resources for all targets before running tests
            for target in &targets {
                renderer.copy_resources(target)?;
            }

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
