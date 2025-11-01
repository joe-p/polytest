use anyhow::Result;
use regex::Regex;

use crate::{render::Renderer, target::Target};

const GROUP_COMMENT: &str = "Polytest Group:";
const SUITE_COMMENT: &str = "Polytest Suite:";

pub fn get_group_comment(group: &str) -> String {
    format!("{} {}", GROUP_COMMENT, group)
}

pub fn get_groups(input: &str) -> Vec<String> {
    let re = Regex::new(format!(r"{} (.*)", GROUP_COMMENT).as_str()).unwrap();
    let mut groups = Vec::new();
    for cap in re.captures_iter(input) {
        groups.push(cap[1].to_string().trim().to_string());
    }
    groups
}

pub fn find_suite(input: &str, name: &str) -> Result<bool> {
    let re = Regex::new(format!(r"{} {}", SUITE_COMMENT, name).as_str()).unwrap();
    Ok(re.is_match(input))
}

pub struct SuiteChunk {
    pub content: String,
    pub start: usize,
    pub end: usize,
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

pub fn find_test(input: &str, target: &Target, name: &str, renderer: &Renderer) -> Result<bool> {
    let re = Regex::new(&renderer.render_test_regex(target, name)?).unwrap();
    Ok(re.is_match(input))
}
