# Multiple Repos

Below is the recommended workflow when an engineering team is working with multiple repositories that all share the same Polytest configuration. This flow is recommended because it avoids the manual overhead of copying configuration changes to multiple repositories or having to work with git submodules.

For this document, we will be assuming we are writing a library called `my-lib` that is implemented in two languages: Python and TypeScript. Each implementation is in its own repo: `my-org/my-lib-py` and `my-org/my-lib-ts`. The polytest configuration will live in a third repository called `my-org/my-lib-polytest`.

## Setup

### Step 1. Create the Polytest Configuration Repo

In this repo, define the Polytest configuration file(s). All the paths within the configuration should be written knowing that the `my-lib-polytest` repo will be a directory in the `my-lib-*` repos.

For example, if the tests in `my-lib-py` live in `tests/polytest_tests`, then the paths in the configuration should be written as `../tests/polytes_tests`.

### Step 2. Add Polytest Test Runner with Git Flag

In each implementation repo, add a script to execute Polytest with the `--git` flag pointing to the Polytest configuration repo

For example, in TypeScript, you might add a script to your `package.json` like this:

```json
"scripts": {
  "polytest": "polytest run --git https://github.com/my-org/my-lib-polytest.git#main run -t vitest"
```

The `#main` at the end of the URL specifies the branch to use. You can change this to point to any branch, tag, or commit hash.

## Workflow: Running Local Tests

When developing, you might not want to push changes to the Polytest configuration repo every time you want to change tests. In this case, you can clone the directory locally and run polytest from there. It is important to ensure the path you clone into is in the root of the implementation repo so that the relative paths in the configuration file work correctly. This path should also be ignored by git.

For example:

**.gitignore**:

```
/my-lib-polytest
```

**clone command**:

```bash
git clone https://github.com/my-org/my-lib-polytest.git
```

**polytest command**:

```bash
polytest --config ./my-lib-polytest/my_suite.json run -t vitest
```

**package.json**:

```json
"scripts": {
  "polytest:dev": "polytest --config ./my-lib-polytest/my_suite.json run -t vitest"
}
```

## Workflow: PRs and Releases

When merging features in the implementation repos, the Polytest command should ideally always point to `main`. This means before merging the feature into the implementation repo, there should be a corresponding PR into `my-lib-polytest` on `main`. It's unreasonable to expect that all feature branches implement the same changes at the same time, so you can take advantage of the `exclude_targets` field update `main` without breaking implementation repos:

```json
 "test": {
        "radius": {
          "desc": "A circle should be able to accurately calculate its radius"
          "exclude_targets": ["python"] // Not yet implemented in Python
        },
```

This, however, may not be viable when there are major breaking changes to the Polytest configuration repo (i.e completely removing tests) that will not be compatible with every implementation repo at the same time. In this case, feature branches can be used to point to specific branches in the Polytest configuration repo that contain the necessary changes:

```json
"scripts": {
  "polytest": "polytest run --git https://github.com/my-org/my-lib-polytest.git#feat!/some_big_breaking_change run -t vitest"
```

Production releases, however, should always point to `main` to ensure stability and feature parity. This can be enforced in CI/CD pipelines by hard-coding the Polytest command in the pipeline configuration:

```yaml
steps:
  - name: Run Polytest
    run: polytest run --git https://github.com/my-org/my-lib-polytest.git#main run -t vitest
```
