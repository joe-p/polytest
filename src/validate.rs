use anyhow::{anyhow, Context, Result};
use regex::Regex;

use crate::parsing::{find_test, get_suite_chunk};
use crate::{target::Target, ConfigMeta, Suite};

pub fn validate_target(
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
                std::path::absolute(suite_file)?.display()
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
                if test.exclude_targets.contains(&target.id) {
                    println!(
                        "test \"{}\" excluded from {} for {}",
                        test.name, suite.name, target.id
                    );
                    continue;
                }

                if !find_test(&contents, target, &test.name, env)? {
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
