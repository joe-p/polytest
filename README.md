# Polytest

Polytest is a language-agnostic tool for generating test scaffolding and keeping test plans in sync across teams and implementations. This tool is designed to be an answer to "how can we be sure engineering teams are implementing the right tests?" and is intended to offer a low-friction way for engineering teams to align their test implementations with their test plans.

## Features

- Define test plans in simple TOML format
- Reuse test cases via test groups and suites
- Validate all test cases are implemented
- Validate no tests are implemented that are not in the test plan
- Generation and validation of any language/framework via minijinja templates
- Out of the box support for Python (`pytest`) and TypeScript (`bun`) test generation and validation
- Out of the box support for markdown test plan generation

## Installation

```bash
cargo install --git https://github.com/joe-p/polytest.git
```

## Usage

### Configuration

Polytest is entirely driven by a TOML configuration file. This file, `polytest.toml` is used to define the test plan and describe the test generation and validation targets. See [example/vehicles/polytest.toml](examples/vehicles/polytest.toml) for how to define a test plan and targets. The generated test scaffolding can be seen for [pytest](examples/vehicles/generated/pytest/) and [bun](examples/vehicles/generated/bun/). This example also contains a [markdown test plan](examples/vehicles/generated/vehicles_example.md) file that is automatically generated from the `polytest.toml`.

### Generate

In any directory with a `polytest.toml` file, run `polytest generate` to generate the test scaffolding for all targets. Any files that do not yet exist will be created and test cases without implementations will have scaffolding generated. Any existing code will not be overwritten.

### Validate

In any directory with a `polytest.toml` file, run `polytest validate` to validate that all test cases are implemented and that no tests are implemented that are not in the test plan.

## FAQs

### Why not use X, Y, or Z?

Polytest is designed to be a **low-friction** tool for engineering teams. Many test frameworks, such as [cucumber](https://cucumber.io/), sound great on paper but often lead to too much abstraction and another layer of complexity to maintain. This results in tests being difficult to write thus poor or missing test implementations. Polytest is intentionally unopinionated about how tests are implemented which allows it to be much more flexible and fit the needs of the engineering team. If a team **wants** to write test cases using [Gherkin](https://cucumber.io/docs/gherkin/reference) keywords they can (they could even write custom Gherkin-based test generators) but it is entirely optional.

Popular test tools (especially those pitched as "BDD" frameworks) also try to promise a way for project managers to easily write tests in natural language that are then programmatically turned into test implementations through the cucumber framework. This, however, can eventually lead to idiosyncrasies between how test cases are written and how they should actually be implemented and the opinionated nature of these frameworks makes these discrepancies difficult to overcome. Instead, Polytest uses a bottom-up approach for syncing test plans between engineering teams and product managers. The `polytest.toml` file can be managed directly within the project repository, but generators can be used to create high-level test plan overviews (i.e. [in markdown](./examples/vehicles/generated/vehicles_example.md)) that are easily reviewable by project managers or other stakeholders. The format of `polytest.toml` is simple enough that anyone should be able to make direct contributions to the test plan.
