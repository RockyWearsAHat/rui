# Git Hooks

This document describes the git hooks installed in this repository.

## Pre-commit Hook (`.git/hooks/pre-commit`)

The pre-commit hook enforces code quality standards before allowing commits.

### Contract

The hook runs the following checks on every commit attempt:

1. **Code Formatting**: `cargo fmt --check`
   - Ensures all Rust code follows the project's formatting standards.
   - Fails if any file would be reformatted by `cargo fmt`.

2. **Linting**: `cargo clippy --all-targets -- -D warnings`
   - Ensures all clippy lints pass with warnings treated as errors.
   - Checks all targets (binary, library, tests, examples, etc.).

### Behavior

- If either check fails, the commit is rejected with the failing check's output.
- If both checks pass, the commit is allowed and the hook exits with code 0.
- The hook is executable (mode: `755`) and located at `.git/hooks/pre-commit`.

### Fixing Violations

If the hook rejects a commit:

- **Formatting**: Run `cargo fmt` to automatically fix formatting issues.
- **Linting**: Run `cargo clippy --fix --all-targets` to automatically fix common clippy issues, or manually address warnings shown by `cargo clippy --all-targets`.

After fixing, stage the changes and attempt the commit again.

### Testing

The hook's contract is verified by tests in `tests/setup.rs`:

- `pre_commit_hook_exists_and_is_executable`: Verifies the hook file exists and is executable.
- `pre_commit_hook_runs_successfully_when_code_is_clean`: Verifies the hook exits with code 0 when the code is clean.

Run the tests with:

```bash
cargo test --test setup
```
