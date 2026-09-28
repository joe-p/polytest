use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::ensure;
use color_eyre::eyre::{eyre, Context, Result};
use indexmap::IndexMap;
use std::env;
use std::fmt::Display;
use std::path::PathBuf;

use crate::config::ConfigMeta;
use crate::render::Renderer;
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
    }

    Ok(())
}
