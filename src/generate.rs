use anyhow::{Context, Result};

use crate::target::Target;
use crate::{
    find_suite, find_test, get_group_comment, get_groups, get_suite_chunk, insert_after_keyword,
    ConfigMeta, Document, Group, Suite, Test,
};

pub fn generate_suite(
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

pub fn generate_document(
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
