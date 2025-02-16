# Polytest Templates

## Minijina Environment

Polytest uses [minijinja](https://docs.rs/minijinja/latest/minijinja/) as a templating engine. If you are familiar with Jinja2, you should feel right at home with the Polytest templates.

The rendering environment used by Polytest has the following options set. Currently Polytest does not support changing these options are modifying the environment, but a feature request is welcome if this is desireable.

### lstrip_blocks

Removes leading spaces and tabs from the start of a line to a block.

### trim_blocks

Removes the first newline after a block.

## Filters

### minijinja

The list of built-in minijinja filters can be seen [here](https://docs.rs/minijinja/2.7.0/minijinja/filters/index.html#functions)

### Polytest

Polytest also provides the following filters which are available to all templates

#### convert_case(case: str)

Converts a string to the given case. The following cases are supported

* Upper
* Lower
* Title
* Sentence
* Toggle
* Camel
* Pascal
* UpperCamel
* Snake
* Constant
* UpperSnake
* Kebab
* Cobol
* UpperKebab
* Train
* Flat
* UpperFlat
* Alternating

For more information on behavior see https://docs.rs/convert_case/0.7.1/convert_case/enum.Case.html

##### Example

```toml
suite_file_name_template = "test_{{ suite.name | convert_case('Snake') }}.rb"
```

## Suite, Group, and Test Templates

The `custom_target.<NAME>.template_dir` configiration option defines the directory that contains minijinja template files that are used to generate the test code scaffolding.

### Suite Template

#### Required Marker

Each suite template must contain a marker that indicates the start of the suite where the group templates will be inserted. Generally this will be a comment, but it can be anything that contains the string `Polytest Suite: {{ suite.name }}`

#### Available Variables

* `suite` - Python representation of the `Suite` struct

#### Example

The following is an example from the `bun` target.

```ts
import { expect, test, describe } from "bun:test";

describe("{{ suite.name }}", () => {
  // Polytest Suite: {{ suite.name }}

});
```

### Group Template

#### Required Marker

Each group template must contain a marker that indicates the start of the group where the test templates will be inserted. Generally this will be a comment, but it can be anything that contains the string `Polytest Group: {{ group.name }}`

#### Available Variables

* `group` - Python representation of the `Group` struct

#### Example

The following is an example from the `bun` target.

```ts


  describe("{{ group.name }}", () => {
    // Polytest Group: {{ group.name }}

  });
```

### Test Template

The test template is the scaffolding for the actual test case implementation. It is generally recommended to generate code that will throw an error or fail indicating that the test case has not been implemented.

#### Available Variables

* `test` - Python representation of the `Test` struct

#### Example

The following is an example from the `bun` target.

```ts


    test("{{ test.name }}", () => {
      throw new Error("TEST NOT IMPLEMENTED");
    });
```

