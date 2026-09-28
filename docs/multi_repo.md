# Multiple Repos

Below is the recommended workflow when an engineering team is working with multiple repositories that all share the same Polytest configuration. This flow uses a git submodule so each implementation repo shares a single source of configuration without copying changes between repositories, while still pinning the exact configuration version it is tested against.

For this document, we will be assuming we are writing a library called `my-lib` that is implemented in two languages: Python and TypeScript. Each implementation is in its own repo: `my-org/my-lib-py` and `my-org/my-lib-ts`. The polytest configuration will live in a third repository called `my-org/my-lib-polytest`.

## Setup

### Step 1. Create the Polytest Configuration Repo

In this repo, define the Polytest configuration file(s). All the paths within the configuration should be written knowing that the `my-lib-polytest` repo will be a directory in the `my-lib-*` repos.

For example, if the tests in `my-lib-py` live in `tests/polytest_tests`, then the paths in the configuration should be written as `../tests/polytes_tests`.

### Step 2. Add the Configuration Repo as a Submodule

In each implementation repo, add the Polytest configuration repo as a git submodule in the root of the implementation repo so the relative paths in the configuration file resolve correctly:

```bash
git submodule add -b main https://github.com/my-org/my-lib-polytest.git my-lib-polytest
```

The `-b main` records `main` as the branch to follow in `.gitmodules`, which lets `git submodule update --remote` pull the latest configuration.

Anyone cloning the implementation repo should clone with submodules:

```bash
git clone --recurse-submodules https://github.com/my-org/my-lib-ts.git
# or, in an existing clone
git submodule update --init
```

### Step 3. Point Polytest at the Submodule

Run Polytest with `--config` pointing at the configuration file inside the submodule. For example, in TypeScript, you might add scripts to your `package.json` like this:

```json
"scripts": {
  "polytest:generate": "polytest --config ./my-lib-polytest/my_suite.json generate -t vitest",
  "polytest:validate": "polytest --config ./my-lib-polytest/my_suite.json validate -t vitest"
}
```

## Workflow: Updating the Configuration

The submodule pins a specific commit of `my-lib-polytest`, so configuration changes only reach an implementation repo when that repo updates the pin. To pull in the latest configuration from `main`:

```bash
git submodule update --remote my-lib-polytest
polytest --config ./my-lib-polytest/my_suite.json generate -t vitest
git add my-lib-polytest
git commit -m "chore: update polytest configuration"
```

When developing, you can edit the configuration inside the submodule directly, commit there, and push those commits to `my-lib-polytest` once they're ready. Remember to commit the updated submodule pointer in the implementation repo afterwards.

## Workflow: PRs and Releases

When merging features in the implementation repos, the submodule should ideally always point to a commit on `main` of `my-lib-polytest`. This means before merging the feature into the implementation repo, there should be a corresponding PR into `my-lib-polytest` on `main`. It's unreasonable to expect that all feature branches implement the same changes at the same time, so you can take advantage of the `exclude_targets` field to update `main` without breaking implementation repos:

```json
 "test": {
        "radius": {
          "desc": "A circle should be able to accurately calculate its radius"
          "exclude_targets": ["python"] // Not yet implemented in Python
        },
```

This, however, may not be viable when there are major breaking changes to the Polytest configuration repo (i.e completely removing tests) that will not be compatible with every implementation repo at the same time. In this case, a feature branch in the implementation repo can pin the submodule to a commit on a feature branch of `my-lib-polytest`:

```bash
git -C my-lib-polytest fetch origin feat!/some_big_breaking_change
git -C my-lib-polytest checkout FETCH_HEAD
git add my-lib-polytest
```

Production releases, however, should always pin a commit that is on `main` to ensure stability and feature parity. This can be enforced in CI/CD pipelines:

```yaml
steps:
  - uses: actions/checkout@v4
    with:
      submodules: true
  - name: Ensure Polytest configuration is on main
    run: |
      git -C my-lib-polytest fetch origin main
      git -C my-lib-polytest merge-base --is-ancestor HEAD origin/main
  - name: Generate Polytest tests
    run: polytest --config ./my-lib-polytest/my_suite.json generate -t vitest
  - name: Validate Polytest tests
    run: polytest --config ./my-lib-polytest/my_suite.json validate -t vitest
```
