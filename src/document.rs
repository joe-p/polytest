use color_eyre::eyre::{Context, ContextCompat, Result};
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct DocumentConfig {
    out_file: PathBuf,
    template: Option<String>,
}

pub struct Document {
    pub out_file: PathBuf,
    pub template: String,
}

impl Document {
    pub fn from_config(config: &DocumentConfig, id: &str, config_root: &Path) -> Result<Self> {
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
