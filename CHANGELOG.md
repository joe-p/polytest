# UNRELEASED: 0.5.0

## Breaking Changes

- Add required `version` field to config that must be in MAJOR.MINOR format (e.g., "0.4") and validates against polytest binary to ensure compatibility

# 0.4.0

## Breaking Changes

- Removed support for TOML in favor of JSON (and JSONC)

## Features

- Add `--config` flag to specify custom config file path
- Add `--git` flag to enable/disable git integration (git is now optional)
- Add support for vitest test framework
- Add ability to dump default targets
- Add `resource_dir` to top-level and target configs to specify a directory that is copied before generating or running tests

## Fixes

- Replace separator with `_` in suite file path
- Escape regex when parsing suite name
- Handle parameterized pytest tests
- Fix typo in JS test regex
- Fix vitest regex
- Use proper relative directory for runner `work_dir`
- Make `out_file` relative to the config root

# 0.3.0

## Features

- Add `exclude_targets` for tests
- Run test runners in parallel by default (disabled via `--no-parallel`)

# 0.2.2

## Fixes

- fix(pytest): convert test name to snake_case when parsing pytest results

# 0.2.1

## Fixes

- Properly handle omitted tables in the config