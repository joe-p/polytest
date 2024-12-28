use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(rename = "suite")]
    pub suites: Vec<Suite>,

    #[serde(rename = "group")]
    pub groups: Vec<Group>,

    #[serde(rename = "test")]
    pub tests: Vec<Test>,
}

impl Config {
    pub fn from_file(path: &str) -> Self {
        let contents = std::fs::read_to_string(path).unwrap();
        toml::from_str(&contents).unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Suite {
    pub id: String,
    pub name: Option<String>,
    pub groups: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Group {
    pub id: String,
    pub tests: Vec<String>,
    pub name: Option<String>,
    pub desc: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Test {
    pub id: String,
    pub name: Option<String>,
    pub desc: Option<String>,
}

fn main() {
    let config = Config::from_file("examples/vehicles/polytest.toml");
    println!("{:?}", config);
    generate_markdown(&config);
    let jinja_value = minijinja::Value::from_serialize(&config);
    let mut env = minijinja::Environment::new();
    env.add_template("markdown", include_str!("../templates/markdown.jinja"))
        .unwrap();
    let template = env.get_template("markdown").unwrap();
    println!(
        "{}",
        template
            .render(minijinja::context! { config => jinja_value })
            .unwrap()
    );
}

fn get_anchor(id: &str) -> String {
    id.replace(" ", "-")
}

fn generate_markdown(config: &Config) {
    let mut markdown = String::new();
    markdown.push_str("# Polytest Test Plan\n");

    markdown.push_str("## Test Suites\n");
    for suite in &config.suites {
        markdown.push_str(&format!("\n### {}\n\n", suite.id));
        markdown.push_str("| Name | Description |\n");
        markdown.push_str("| --- | --- |\n");
        for group_id in &suite.groups {
            let group = config
                .groups
                .iter()
                .find(|g| g.id == *group_id)
                .expect("group should exist");

            let name = group.name.as_deref().unwrap_or(&group.id);

            markdown.push_str(&format!(
                "| [{}](#{}) | {} |\n",
                name,
                get_anchor(name),
                group.desc.as_deref().unwrap_or("")
            ));
        }
    }

    markdown.push_str("\n## Test Groups\n");
    for group in &config.groups {
        markdown.push_str(&format!("\n### {}\n\n", group.id));
        markdown.push_str("| Name | Description |\n");
        markdown.push_str("| --- | --- |\n");
        for test_id in &group.tests {
            let test = config
                .tests
                .iter()
                .find(|t| t.id == *test_id)
                .unwrap_or_else(|| panic!("test {} should exist", test_id));

            let name = test.name.as_deref().unwrap_or(&test.id);
            markdown.push_str(&format!(
                "| [{}](#{}) | {} |\n",
                name,
                get_anchor(name),
                test.desc.as_deref().unwrap_or("")
            ));
        }
    }

    markdown.push_str("\n## Test Cases\n");
    for test in &config.tests {
        let name = test.name.as_deref().unwrap_or(&test.id);
        markdown.push_str(&format!("\n### {}\n\n", name));
        if let Some(desc) = &test.desc {
            markdown.push_str(&format!("{}\n", desc));
        }
    }

    std::fs::write("examples/vehicles/generated_markdown.md", markdown).unwrap();
}
