# UNRELEASED: 0.4.0

## Features

- Add support for JSONC (JSON with comments)
- Add `--config` flag to specify custom config file path
- Add `--git` flag to enable/disable git integration (git is now optional)
- Add support for vitest test framework
- Add ability to dump default targets
- Support JSON config format (in addition to TOML)

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
