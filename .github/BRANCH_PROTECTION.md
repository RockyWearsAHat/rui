# Branch Protection Rules Configuration

This document specifies the GitHub branch protection rules for the `rui` project's `main` branch.

## Overview

The `main` branch is protected to ensure all code merges have passed:
- Automated testing (CI/CD)
- Code review
- Status checks
- Conflict resolution

This prevents accidental breakage and ensures code quality.

## Required Status Checks

The following GitHub Actions workflows must pass before merging:

### 1. Core Testing (`ci` workflow)
- **Scope**: All platforms (macOS, Linux, Windows)
- **Requirements**:
  - `test on ubuntu-latest` ✅
  - `test on macos-latest` ✅
  - `test on windows-latest` ✅
  - `test WASM target` ✅
  - `lint` ✅

### 2. Recipe 2 Verification (`recipe-2-verification` workflow)
- **Scope**: Backend implementation verification (Phase 1-3)
- **Requirements**:
  - `Phase 1: Compilation [macOS]` ✅
  - `Phase 1: Compilation [Linux (X11)]` ✅
  - `Phase 1: Compilation [Linux (x86_64)]` ✅
  - `Phase 1: Compilation [Windows]` ✅
  - `Phase 1: Compilation [WASM (wasm32)]` ✅
  - `Phase 2: Integration [macOS]` ✅
  - `Phase 2: Integration [Linux (X11)]` ✅
  - `Phase 2: Integration [Windows]` ✅
  - `Phase 3: Parity [macOS]` ✅
  - `Phase 3: Parity [Linux (X11)]` ✅
  - `Phase 3: Parity [Windows]` ✅
  - `Verify All Phases Complete` ✅

## Setup Instructions

### Enable on GitHub

1. Navigate to: `Settings` → `Branches` → `Branch protection rules`
2. Click `Add rule`
3. Configure:
   - **Branch name pattern**: `main`
   - **Protect matching branches**: ✓ Checked

### Required status checks to pass before merging

Under "Require status checks to pass before merging":

1. ✓ **Require branches to be up to date before merging**
2. ✓ **Require status checks to pass before merging**

Select the following status checks:

**From `ci` workflow:**
- `test on ubuntu-latest`
- `test on macos-latest`
- `test on windows-latest`
- `test WASM target`
- `lint`

**From `recipe-2-verification` workflow:**
- `Phase 1: Compilation [macOS]`
- `Phase 1: Compilation [Linux (X11)]`
- `Phase 1: Compilation [Linux (x86_64)]`
- `Phase 1: Compilation [Windows]`
- `Phase 1: Compilation [WASM (wasm32)]`
- `Phase 2: Integration [macOS]`
- `Phase 2: Integration [Linux (X11)]`
- `Phase 2: Integration [Windows]`
- `Phase 3: Parity [macOS]`
- `Phase 3: Parity [Linux (X11)]`
- `Phase 3: Parity [Windows]`
- `Verify All Phases Complete`

### Other Protection Rules

Under "Other branch protection settings":

1. ✓ **Dismiss stale pull request approvals when new commits are pushed**
2. ✓ **Require code reviews before merging**
   - Number of required reviews: `1`
3. ✓ **Require approval of the most recent reviewable push**
4. ✓ **Restrict who can push to matching branches** (optional)
   - Allow pushes from: `Administrators`

### Enforcement

- ✓ **Include administrators** (admins follow the same rules)
- ✓ **Restrict who can push to matching branches** (GitHub admins only)

## Performance Regression Detection

The `performance-regression` job in the `recipe-2-verification` workflow:

1. **Builds release profile** (`cargo build --release`)
2. **Runs benchmarks** (if present in `benches/`)
3. **Checks binary size** (tracks unintended bloat)

### Performance Baseline

Create or update baselines manually:

```bash
# Record current performance baseline
cargo bench --bench main -- --save-baseline current
```

### CI Integration

The workflow captures:
- Binary size (flagged if >10% growth)
- Build time
- Test execution time

These are logged for trend analysis.

## Wayland Support

When Wayland backend is enabled:

- Add to `Phase 1: Compilation`:
  ```
  - platform: "Linux (Wayland)"
    runs-on: ubuntu-latest
    install: "sudo apt-get update && sudo apt-get install -y libwayland-dev libxkbcommon-dev"
    features: "wayland"
  ```

- Add to `Phase 2` and `Phase 3` accordingly

Update this file when Wayland support is added.

## Cross-Compilation Targets

Future targets to add:

- **iOS**: `aarch64-apple-ios` (when implemented)
- **Android**: `aarch64-linux-android` (when implemented)
- **RISC-V**: `riscv64gc-unknown-linux-gnu` (when implemented)

## Maintenance

This configuration should be reviewed:
- After each new platform backend is added
- When new test suites are created
- Monthly for performance baseline updates

---

**Last Updated**: 2026-08-31
**Related Documents**: STEP_13_RECIPE_2_VERIFICATION.md, .github/workflows/
