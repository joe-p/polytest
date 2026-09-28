use clap::{Args, Parser, Subcommand};
use color_eyre::eyre::{eyre, Context, Result};
use indexmap::IndexMap;
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
    /// Path to the config file (supports .json and .jsonc)
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

fn main() -> Result<()> {
    color_eyre::install()?;

    let parsed = Cli::parse();

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
