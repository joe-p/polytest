use color_eyre::eyre::{eyre, Context, Result};
use regex::Regex;

use crate::parsing::{find_test, get_suite_chunk};
use crate::render::Renderer;
use crate::{target::Target, ConfigMeta, Suite};

pub fn validate_target(
    config_meta: &ConfigMeta,
    target: &Target,
    renderer: &Renderer,
) -> Result<()> {
    let suites: Vec<Suite> = config_meta
        .config
        .suites
        .iter()
        .map(|(id, s)| Suite::from_config(&config_meta.config, s, id))
        .collect();

    for suite in &suites {
        let suite_file_name = renderer.render_suite_file_name(target, suite)?;

        let suite_file = target.out_dir.join(&suite_file_name);
        if !suite_file.exists() {
            return Err(eyre!(
                "suite file {} does not exist",
                std::path::absolute(suite_file)?.display()
            ));
        }

        let contents = std::fs::read_to_string(&suite_file).context(format!(
            "failed to read existing suite file for {}",
            target.id
        ))?;

        let suite_chunk = get_suite_chunk(&contents, &suite.name)?;

        let all_tests_regex = Regex::new(&renderer.render_all_tests_regex(target)?).unwrap();

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

                if !find_test(&contents, target, &test.name, renderer)? {
                    return Err(eyre!(
                        "test \"{}\" does not exist in {}",
                        test.name,
                        suite_file.display()
                    ));
                } else {
                    let test_regex = renderer.render_test_regex(target, &test.name)?;

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
            return Err(eyre!(
                "found test implementation(s) in \"{}\" suite in {} that were not defined in the test plan\n{}",
                suite.name,
                suite_file.display(),
                remaining_tests.join("\n")
            ));
        }
    }

    Ok(())
}
