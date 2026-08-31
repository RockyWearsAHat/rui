# Deployment & Release Guide

## Overview

This guide documents how to publish rui-native releases to crates.io and manage the release process.

## Release Workflow

### Step 1: Prepare Release

1. **Update Version** in `Cargo.toml`:
```toml
[package]
name = "rui-native"
version = "0.2.0"  # Increment from 0.1.0
```

2. **Update CHANGELOG.md** (if you maintain one):
```markdown
# v0.2.0 (2026-09-15)

## New Features
- Feature 1
- Feature 2

## Bug Fixes
- Fix 1
- Fix 2

## Breaking Changes
None
```

3. **Verify all tests pass**:
```bash
cargo test
cargo test --test setup
```

4. **Update documentation** if APIs changed:
```bash
cargo doc --no-deps --open
# Review docs for completeness
```

### Step 2: Dry-Run Publish

**Always test publication before committing:**

```bash
cargo publish --dry-run
```

**Expected output:**
```
Uploading rui-native v0.2.0 to registry `crates-io`
Uploading rui-native v0.2.0 to registry `crates-io`
Verifying rui-native v0.2.0 ...
Compiling rui-native v0.2.0
Finished `release` profile [optimized] target(s) in 1.23s
```

**Common errors and fixes:**

| Error | Cause | Fix |
|-------|-------|-----|
| `error: "rui-native" not found in registry` | Package not yet created | First publish must use `cargo publish` (not dry-run) |
| `error: crate version 0.2.0 already published` | Version exists on crates.io | Increment version in Cargo.toml |
| `error: missing required metadata: description` | Cargo.toml missing field | Add `description = "..."` to `[package]` |
| `error: missing required metadata: license` | Cargo.toml missing license | Add `license = "MIT"` or `license = "MIT OR Apache-2.0"` |

### Step 3: Commit Version Changes

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: bump version to 0.2.0"
git push origin main
```

### Step 4: Create GitHub Release

```bash
# Create tag
git tag v0.2.0
git push origin v0.2.0

# Create release on GitHub (via CLI or web)
gh release create v0.2.0 \
  --title "rui-native v0.2.0" \
  --notes "Release notes here"
```

### Step 5: Publish to crates.io

**Prerequisites:**
- Rust installed (`rustup update`)
- crates.io account created (https://crates.io)
- `CARGO_TOKEN` configured (see Setup section below)

**Publish:**
```bash
cargo publish
```

**Verify publication:**
```bash
# Wait 5-10 seconds for crates.io to index
curl https://crates.io/api/v1/crates/rui-native/0.2.0 | jq .

# Or visit: https://crates.io/crates/rui-native/0.2.0
# And: https://docs.rs/rui-native/0.2.0/
```

---

## Setup for Maintainers

### Initial Setup (One-Time)

#### 1. Create crates.io Account

1. Visit https://crates.io/me
2. Sign in with GitHub
3. Create account
4. Verify email

#### 2. Get API Token

1. Visit https://crates.io/me
2. Under "API Tokens", click "New Token"
3. Copy token (shown once)

#### 3. Configure Local Machine

**Option A: Local configuration (less secure, for personal use)**

```bash
cargo login
# Paste token when prompted
# Writes to ~/.cargo/credentials.toml
```

**Option B: GitHub Actions (recommended for CI/CD)**

```bash
# Add secret to GitHub repository
gh secret set CARGO_TOKEN --body "YOUR_TOKEN_HERE"
```

Then in `.github/workflows/publish.yml`:

```yaml
- name: Publish to crates.io
  env:
    CARGO_TOKEN: ${{ secrets.CARGO_TOKEN }}
  run: cargo publish --token "$CARGO_TOKEN"
```

### Verify Setup

```bash
# Check token is stored
cat ~/.cargo/credentials.toml  # (local setup)

# Or verify CI/CD secret exists
gh secret list
```

---

## Automated Publishing via CI/CD

The repository includes `.github/workflows/publish.yml` for automated publication.

### How It Works

1. **Trigger:** When a GitHub release is created/published
2. **Action:** Workflow runs `cargo publish`
3. **Auth:** Uses `CARGO_TOKEN` from GitHub Secrets
4. **Result:** Package published to crates.io

### Setup Automated Publishing

1. **Add CARGO_TOKEN to GitHub Secrets:**

```bash
gh secret set CARGO_TOKEN --body "YOUR_TOKEN"
```

2. **Verify workflow file exists:**

```bash
cat .github/workflows/publish.yml
```

Expected content:
```yaml
name: Publish to crates.io

on:
  release:
    types: [published]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish --token "${{ secrets.CARGO_TOKEN }}"
```

3. **Create a release to trigger:**

```bash
gh release create v0.2.0 --title "rui-native v0.2.0"
# Workflow runs automatically
```

---

## Pre-Release Checklist

Before publishing any version:

### Code Quality
- [ ] `cargo test` passes (all platforms)
- [ ] `cargo clippy` clean (zero warnings)
- [ ] `cargo fmt --check` passes
- [ ] No uncommitted changes
- [ ] All branches merged to main

### Documentation
- [ ] API docs complete (`cargo doc --open`)
- [ ] Examples all work (`cargo run -p rui --example *`)
- [ ] README updated
- [ ] CHANGELOG updated (if maintained)
- [ ] No broken links in docs

### Testing
- [ ] All tests passing
- [ ] Integration tests pass
- [ ] WASM tests pass (if applicable)
- [ ] Example builds work
- [ ] Release build tested

### Metadata
- [ ] Version bumped in Cargo.toml
- [ ] License field set
- [ ] Description field set
- [ ] Repository field correct
- [ ] Documentation field set (if external docs)

### Publishing
- [ ] Dry-run publish succeeds
- [ ] No unexpected files included
- [ ] File count reasonable (< 500 files)
- [ ] Package size reasonable (< 10 MiB)

---

## Troubleshooting

### "Crate already published"

**Symptom:** Error "crate version X.X.X already published"

**Cause:** Version exists on crates.io

**Solution:** 
```bash
# Increment version in Cargo.toml
cargo publish --dry-run
cargo publish
```

### "Unauthorized" or token error

**Symptom:** `error: failed to fetch repository: 401 Unauthorized`

**Cause:** Missing or invalid CARGO_TOKEN

**Solution:**
```bash
# Refresh token from crates.io
cargo login
# Or set CARGO_TOKEN env var
export CARGO_TOKEN="your_token_here"
cargo publish
```

### "Cannot publish yanked version"

**Symptom:** `error: crate version was previously yanked`

**Cause:** Version was yanked on crates.io (hidden due to problems)

**Solution:**
```bash
# Un-yank via web UI: https://crates.io/crates/rui-native
# Or use: cargo yank --vers 0.2.0 --undo
```

### Large upload takes long time

**Symptom:** Publish hangs at "Uploading..."

**Cause:** Network slow, or crates.io under load

**Solution:**
```bash
# Retry with timeout
timeout 180 cargo publish --token "$CARGO_TOKEN"
```

### Documentation not updating on docs.rs

**Symptom:** Old documentation still shows at https://docs.rs/rui-native/

**Cause:** docs.rs caches; new version needs time to build

**Solution:**
1. Visit https://docs.rs/releases/search?query=rui-native
2. Click "Retry" if build failed
3. Wait 5–10 minutes for new version to build
4. Version usually available within 10 minutes

---

## Versioning Strategy

### Semantic Versioning

Follow [semver.org](https://semver.org/):

```
MAJOR.MINOR.PATCH (e.g., 0.1.0)

MAJOR: Breaking changes (API changes, removed functions)
MINOR: New features (backward compatible)
PATCH: Bug fixes (backward compatible)
```

### Current Status

- **Current version:** 0.1.0 (pre-1.0 means API unstable)
- **Stability:** Experimental (breaking changes possible in 0.2, 0.3, etc.)
- **v1.0 target:** After API stabilizes and platform support expands

### Example Progression

```
v0.1.0 — Initial release (macOS, Windows, Linux, WASM)
v0.2.0 — Add Wayland support (MINOR bump)
v0.2.1 — Fix X11 input bug (PATCH bump)
v0.3.0 — Redesign color system (MAJOR breaking change, pre-1.0)
v0.4.0 — Add widget library (MINOR bump)
v1.0.0 — API stable, production ready (MAJOR, signals stability)
```

---

## Maintenance After Release

### Monitoring

After publishing:

1. **Check crates.io:**
   - https://crates.io/crates/rui-native
   - Verify metadata (description, license, keywords)

2. **Check docs.rs:**
   - https://docs.rs/rui-native/latest/
   - Verify documentation renders correctly

3. **Monitor GitHub Issues:**
   - Filter by release label
   - Triage bugs and feature requests

### Security Updates

If a security issue is found:

1. **Fix in main branch**
2. **Backport to last stable version** (if needed)
3. **Publish PATCH version immediately**
4. **Announce via security advisory**

Example:
```bash
# On v0.1.0 branch
git checkout v0.1.0
git checkout -b security/x11-event-validation
# Fix issue
git commit -m "fix: validate X11 event source"
git tag v0.1.1
cargo publish --token "$CARGO_TOKEN"
```

### Yanking Broken Versions

If a release is broken:

```bash
# Hide from crates.io
cargo yank --vers 0.1.0

# Later, if fixed:
cargo yank --vers 0.1.0 --undo
```

---

## Release Cadence

### Target Schedule

- **Patch releases:** As needed (security/critical bugs)
- **Minor releases:** Every 2–3 months (features, improvements)
- **Major releases:** As API stabilizes (v1.0)

### Announcement

Each release should be announced via:

1. **GitHub Release notes** (for developers)
2. **crates.io changelog** (included in release notes)
3. **Project discussions** (if community exists)
4. **Reddit r/rust** (major releases only)

---

## Continuous Integration

### CI/CD Pipeline

The project includes `.github/workflows/ci.yml` which runs on every push:

```yaml
- Rust toolchain setup
- cargo test (all tests)
- cargo clippy (linting)
- cargo fmt --check (formatting)
```

**Must pass before merging to main.**

### Pre-Publication Checks

Before triggering publish workflow:

```bash
# Run locally
cargo test
cargo clippy --all-targets
cargo fmt --check
cargo publish --dry-run
```

---

## Rollback Procedure

If a release has critical issues:

### Option 1: Yank (Hide) Version

```bash
cargo yank --vers 0.2.0
# Version no longer recommended; users on 0.1.0 won't auto-upgrade
```

### Option 2: Publish Patch Fix

```bash
# Fix issue
cargo publish  # Publishes 0.2.1
# Communicate via GitHub issue: "Upgrade to 0.2.1"
```

### Option 3: Revert Main + Retract Release

```bash
git revert <commit-hash>
git push origin main
cargo yank --vers 0.2.0
# GitHub release stays but is marked "pre-release: false"
```

---

## FAQ

**Q: Can I push directly to crates.io without GitHub?**
A: Yes. Just run `cargo publish` locally (requires CARGO_TOKEN).

**Q: How do I unlist a version?**
A: Use `cargo yank --vers 0.2.0`. It stays published but isn't recommended.

**Q: Can I change a version after publishing?**
A: No. Yank and publish a new version instead.

**Q: How long until docs.rs updates?**
A: Usually 5–10 minutes. Click "Retry" at https://docs.rs if build fails.

**Q: What if crates.io is down?**
A: Wait. Crates.io is highly available (99.9%+ uptime). Retry after 1 hour.

---

## Support

For questions about:
- **crates.io:** https://crates.io/data/
- **docs.rs:** https://docs.rs/
- **Cargo publishing:** https://doc.rust-lang.org/cargo/reference/publishing.html
- **This project:** https://github.com/RockyWearsAHat/rui/issues

---

**Document version:** 1.0 (2026-08-30)  
**Applies to:** rui-native v0.1.0+
