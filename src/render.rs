use color_eyre::eyre::{eyre, Context, ContextCompat, Result};
use convert_case::{Case, Casing};

use crate::parsing::{find_suite, find_test, get_group_comment, get_groups, get_suite_chunk};
use crate::runner::Runner;
use crate::target::Target;
use crate::ConfigMeta;
use crate::{document::Document, group::Group, suite::Suite, test::Test};

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
        _ => Err(eyre!(
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

pub fn insert_after_keyword(original: &str, to_insert: &str, keyword: &str) -> String {
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

pub struct Renderer {
    env: minijinja::Environment<'static>,
    config_meta: ConfigMeta,
}

impl Renderer {
    pub fn new(targets: &[Target], config_meta: ConfigMeta) -> Result<Self> {
        let mut env = minijinja::Environment::new();
        env.add_filter("convert_case", convert_case_filter);
        env.set_lstrip_blocks(true);

        env.set_trim_blocks(true);

        for (document_id, document_config) in &config_meta.config.documents {
            let document =
                Document::from_config(document_config, document_id, &config_meta.root_dir)?;
            env.add_template_owned(format!("{}_document", document_id), document.template)
                .context(format!(
                    "failed to add document template for document {}",
                    document_id
                ))?;
        }

        for target in targets {
            for (runner_id, runner) in &target.runners {
                let target_runner = target.id.clone() + runner_id;

                env.add_template_owned(
                    format!("{}_fail_regex", target_runner),
                    runner.fail_regex_template.clone(),
                )
                .context(format!(
                    "failed to add fail_regex template for runner {} of target {}",
                    runner_id, target.id
                ))?;

                env.add_template_owned(
                    format!("{}_pass_regex", target_runner),
                    runner.pass_regex_template.clone(),
                )
                .context(format!(
                    "failed to add pass_regex template for runner {} of target {}",
                    runner_id, target.id
                ))?;
            }

            env.add_template_owned(
                format!("{}_suite_file_name", target.id),
                target.suite_file_name_template.clone(),
            )
            .context(format!(
                "failed to add suite_file_name template for target {}",
                target.id
            ))?;

            env.add_template_owned(
                format!("{}_test_regex", target.id),
                target.test_regex_template.clone(),
            )
            .context(format!(
                "failed to add test_regex template for target {}",
                target.id
            ))?;

            env.add_template_owned(
                format!("{}_suite", target.id),
                target.suite_template.clone(),
            )
            .context(format!(
                "failed to add suite template for target {}",
                target.id
            ))?;

            env.add_template_owned(
                format!("{}_group", target.id),
                target.group_template.clone(),
            )
            .context(format!(
                "failed to add group template for target {}",
                target.id
            ))?;

            env.add_template_owned(format!("{}_test", target.id), target.test_template.clone())
                .context(format!(
                    "failed to add test template for target {}",
                    target.id
                ))?;
        }

        Ok(Self { env, config_meta })
    }

    pub fn generate_suite(&self, target: &Target) -> Result<()> {
        let config = &self.config_meta.config;

        let suite_values: Vec<Suite> = config
            .suites
            .iter()
            .map(|(id, s)| Suite::from_config(config, s, id))
            .collect();

        let suite_template_name = format!("{}_suite", target.id);
        let suite_template = self
            .env
            .get_template(suite_template_name.as_str())
            .expect("suite template should have been adedd by generate_target");

        let group_template_name = format!("{}_group", target.id);
        let group_template = self
            .env
            .get_template(group_template_name.as_str())
            .expect("group template should have been adedd by generate_target");

        let test_template_name = format!("{}_test", target.id);
        let test_template = self
            .env
            .get_template(test_template_name.as_str())
            .expect("test template should have been adedd by generate_target");

        for suite in &suite_values {
            let suite_file_name = self.render_suite_file_name(target, suite)?;

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
                            package_name => minijinja::Value::from(&config.package_name),
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
                    if test.exclude_targets.contains(&target.id) {
                        println!(
                            "test \"{}\" excluded from {} for {}",
                            test.name, suite.name, target.id
                        );
                        continue;
                    }

                    // We don't need to get a group-specific chunk because two groups can't have the same test
                    if find_test(&suite_chunk.content, target, &test.name, self)? {
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
                            suite_name => minijinja::Value::from(&suite.name),
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

    pub fn generate_document(&self, doc_id: &str) -> Result<()> {
        let document_config = self
            .config_meta
            .config
            .documents
            .get(doc_id)
            .context(format!("document {} does not exist", doc_id))?;

        let document = Document::from_config(document_config, doc_id, &self.config_meta.root_dir)?;

        let config = &self.config_meta.config;
        let suite_values: Vec<Suite> = config
            .suites
            .iter()
            .map(|(id, s)| Suite::from_config(config, s, id))
            .collect();

        let group_values: Vec<Group> = config
            .groups
            .iter()
            .map(|(id, g)| Group::from_config(g, id))
            .collect();

        let test_values: Vec<Test> = config
            .groups
            .iter()
            .flat_map(|(_, g_cfg)| {
                g_cfg
                    .tests
                    .iter()
                    .map(|(t_id, t_cfg)| Test::from_config(t_cfg, t_id))
            })
            .collect();

        let template = self
            .env
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

    pub fn render_test_regex(&self, target: &Target, test_name: &str) -> Result<String> {
        let test_regex_template_name = format!("{}_test_regex", target.id);
        let test_regex_template =
            self.env
                .get_template(&test_regex_template_name)
                .context(format!(
                    "template should exist since it was just added above for target {}",
                    target.id
                ))?;

        let rendered = test_regex_template
            .render(minijinja::context! {
                name => minijinja::Value::from(test_name),
            })
            .context(format!(
                "failed to render test regex for target {}",
                target.id
            ))?;

        Ok(rendered)
    }

    pub fn render_suite_file_name(&self, target: &Target, suite: &Suite) -> Result<String> {
        let file_template_name = format!("{}_suite_file_name", target.id);
        let file_template = self
            .env
            .get_template(&file_template_name)
            .context(format!("failed to get file template for {}", target.id))?;

        let modified_suite = Suite {
            name: suite.name.replace(std::path::MAIN_SEPARATOR_STR, "_"),
            groups: suite.groups.clone(),
        };

        let rendered = file_template
            .render(minijinja::context! {
                suite => minijinja::Value::from_serialize(modified_suite),
            })
            .context(format!("failed to render file name for {}", target.id))?;

        Ok(rendered)
    }

    pub fn render_all_tests_regex(&self, target: &Target) -> Result<String> {
        let test_regex_template_name = format!("{}_test_regex", target.id);
        let test_regex_template =
            self.env
                .get_template(&test_regex_template_name)
                .context(format!(
                    "template should exist since it was just added above for target {}",
                    target.id
                ))?;

        let rendered = test_regex_template
            .render(minijinja::context! {
                name => minijinja::Value::from(".*"),
            })
            .context(format!(
                "failed to render all tests regex for target {}",
                target.id
            ))?;

        Ok(rendered)
    }

    pub fn render_cmd(&self, runner: &Runner) -> Result<String> {
        Ok(self.env.render_str(
            &runner.command,
            minijinja::context! {
                package_name => minijinja::Value::from(&self.config_meta.config.package_name),
            },
        )?)
    }

    pub fn render_pass_regex(
        &self,
        target_runner: &str,
        suite_file_name: &str,
        suite: &Suite,
        group: &Group,
        test: &Test,
    ) -> Result<String> {
        let template = self
            .env
            .get_template(format!("{}_pass_regex", target_runner).as_str())?;

        template
            .render(minijinja::context! {
                file_name => minijinja::Value::from(suite_file_name),
                suite_name => minijinja::Value::from(&suite.name),
                group_name => minijinja::Value::from(&group.name),
                test_name => minijinja::Value::from(&test.name),
            })
            .context(format!("failed to render pass regex for {}", target_runner))
    }

    pub fn render_fail_regex(
        &self,
        target_runner: &str,
        suite_file_name: &str,
        suite: &Suite,
        group: &Group,
        test: &Test,
    ) -> Result<String> {
        let template = self
            .env
            .get_template(format!("{}_fail_regex", target_runner).as_str())?;

        template
            .render(minijinja::context! {
                file_name => minijinja::Value::from(suite_file_name),
                suite_name => minijinja::Value::from(&suite.name),
                group_name => minijinja::Value::from(&group.name),
                test_name => minijinja::Value::from(&test.name),
            })
            .context(format!("failed to render fail regex for {}", target_runner))
    }
}
