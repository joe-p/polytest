# Polytest Configuration

Polytest is configured via a TOML configuration file named `polytest.toml` by default. All Polytest configuration, including test suite definitions, is done via this file. This file is the one source of truth for Polytest, ensuring there is no confusion about the current state of the test plan.

## name

The test plan is named via the `name` field. This is generally only used for display purposes, but can also be used during document generation.

### Example

```toml
name = "Shapes Test Plan
```

## suite.\<NAME>

Test suites are a collection of tests. For most implementations this roughly maps to the tests in a single file. Test cases are defined under the `suite` table.

### Fields

#### desc

The `desc` field within a suite table is used to describe the suite.

#### groups

The `groups` field within a suite table is used to define the test groups that belong to the suite.

### Example

```toml
[suite.circle]
desc = "A circle is a shape defined by all points on a plane that are equidistant from a given point"
groups = ["shape", "circle"]

[suite.rectangle]
desc = "A rectangle is a polygon with four right angles"
groups = ["shape", "polygon", "rectangle"]

[suite.triangle]
desc = "A triangle is a polygon with three edges"
groups = ["shape", "polygon"]
```

## group.\<NAME>

Groups are a collection of test cases that typically share something in common. You could also think of a group as a label on a test case, but there is a strict "one group" -> "many testcases" relationship.

### Fields

#### desc

The `desc` field within a group table is used to describe the group.

### Example

```toml
[group.polygon]
desc = "Tests that only apply to polygons"

[group.circle]
desc = "Tests that only apply to circles"

[group.shape]
desc = "Tests that apply to any shape. A shape is a polygon OR a circle"

[group.rectangle]
desc = "Tests that apply only to rectangles, a subset of polygons"
```

## test.\<NAME>

Test cases are defined under the `test` table. These map to one test case that will have a pass or fail status when ran.

### Fields

#### desc

The `desc` field describes what is being tested.

#### group

The `group` field is used to define the group that the test case belongs to.

### Example

```toml
[test.area]
desc = "A shape should be able to accurately calculate its area"
group = "shape"

[test.perimeter]
desc = "A shape should be able to accurately calculate its perimeter (or circumference)"
group = "shape"

[test.vertex_count]
desc = "A polygon should accurately count the number of verticies it contains"
group = "polygon"

[test.edge_count]
desc = "A polygon should accurately count the number of edges it contains"
group = "polygon"

[test.is_square]
desc = "A rectangle should be able to accurately determine if it is a square"
group = "rectangle"

[test.radius]
desc = "A circle should be able to accurately calculate its radius"
group = "circle"

[test.diameter]
desc = "A circle should be able to accurately calculate its diameter"
group = "circle"
```

## target.\<NAME>

Test targets are defined under the `target` table. Test targets are the testting frameworks for the implementations (i.e. `pytest` for python).

### Supported Targets

These are the targets supported by Polytest out of the box. Custom targets can also be defined.

* `pytest`
* `bun`

### Fields

#### out_dir

The `out_dir` field is used to define the output directory for the generated test scaffolding and is used as the working directory when the test command is ran.

### Examples

```toml
[target.pytest]
out_dir = "./implementations/python/tests"

[target.bun]
out_dir = "./implementations/bun/__tests__"
```

## custom_target.\<NAME>

Custom test targets can be defined under the `custom_target` table. Custom targets give you full control of scaffolding templates, test execution, and parsing.

### Fields

#### out_dir

The `out_dir` field is used to define the output directory for the generated test scaffolding and is used as the working directory when the test command is ran. The path is relative to the location of the configuration file.

#### suite_file_name_template

The `suite_file_name_template` field is used to define the template for the suite file name. This string is a minijinja template.

##### Template variables

The variables available for use in the template. See [templates documentation](./templates.md) for more information on how these variables can be used.

* `suite` - [Suite](./templates.md#suite) struct

#### test_regex_template

The `test_regex_template` field is used to define regex that Polytest can use to find test implementations in the suite file(s). This string is a minijinja template.

##### Template variables

The variables available for use in the template. See [templates documentation](./templates.md) for more information on how these variables can be used.

* `name` - The name of the test case (i.e. for `test.some_test`, `some_test` )

#### template_dir

The `template_dir` field is used to define the directory that contains the templates for the test target. This directory is used to find the templates for the test target. The path is relative to the location of the configuration file. This directory must contain one file that matches each of the following globs:

* `suite*`
* `group*`
* `test*`

See [templates documentation](./templates.md) for more information on the expected contents of these files and available variables.

## custom_target.\<NAME>.runner

`runner` is a table that defines how to run the test suites and parse the results.

### Fields

#### command

The `command` field is used to define the command to run the test. This should just be the name of the executable.

#### args

The `args` field is used to define the arguments to pass to the test command.

#### fail_regex_template

The `fail_regex_template` field is used to define regex that Polytest can use on the output to determine if a test has failed. This string is a minijinja template.

##### Template variables

The variables available for use in the template. See [templates documentation](./templates.md) for more information on how these variables can be used.

* `file_name` - The name of the file that contains the test (this is the rendered `suite_file_name_template`)
* `suite_name` - The name of the suite that contains the test (i.e. for `suite.some_suite`, `some_suite` )
* `group_name` - The name of the group that contains the test (i.e. for `group.some_group`, `some_group` )
* `test_name` - The name of the test (i.e. for `test.some_test`, `some_test` )

#### pass_fail_regex_template

The `pass_fail_regex_template` field is used to define regex that Polytest can use on the output to determine if a test has passed. This string is a minijinja template.

##### Template variables

The variables available for use in the template. See [templates documentation](./templates.md) for more information on how these variables can be used.

* `file_name` - The name of the file that contains the test (this is the rendered `suite_file_name_template`)
* `suite_name` - The name of the suite that contains the test (i.e. for `suite.some_suite`, `some_suite` )
* `group_name` - The name of the group that contains the test (i.e. for `group.some_group`, `some_group` )
* `test_name` - The name of the test (i.e. for `test.some_test`, `some_test` )

## document.\<NAME>

A document is a generated file that is regenered each time `polytest generate` is ran.

### Fields

#### out_file

The path to the file that will be generated. The path is relative to the location of the configuration file.

#### template

If not given, use the default template for the given name. If given, it is the path to the template file. The path is relative to the location of the configuration file.

##### Template Variables

* `name` - The name of the Polytest plan
* `suites` - A list of all `Suite` structs in the test plan
* `groups` - A list of all `Group` structs in the test plan
* `tests` - A list of all `Test` structs in the test plan

### Example

```toml
[document.markdown]
out_file = "./documents/plan.md"

[document.test_cases_csv]
out_file = "./documents/test_cases.csv"
template = "./templates/test_cases.csv.jinja"
```

### Example Template

```
# Polytest Test Plan
## Test Suites
{% for suite in suites %}

### {{ suite.name | convert_case('Title') }}

| Name | Description |
| --- | --- |
{% for group in suite.groups %}
| [{{ group.name }}](#{{ group.name | convert_case('Kebab') }}) | {{ group.desc }} |
{% endfor %}
{% endfor %}

## Test Groups
{% for group in groups %}

### {{ group.name | convert_case('Title') }}

| Name | Description |
| --- | --- |
{% for test in group.tests %}
| [{{ test.name }}](#{{ test.name | convert_case('Kebab') }}) | {{ test.desc }} |
{% endfor %}
{% endfor %}

## Test Cases
{% for test in tests %}

### {{ test.name }}

{{ test.desc }}
{% endfor %}
```
