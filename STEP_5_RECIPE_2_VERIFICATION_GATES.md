# Recipe 2: X11 Backend — Verification Gates

Reference document for Recipe 2 (X11 Backend Implementation). This file documents the acceptance criteria and test commands for each phase of the three-phase implementation pattern.

## Overview

X11 backend implementation follows a three-phase pattern:
1. **Phase 1: Foundation** — Basic window and event loop
2. **Phase 2: Enhancement** — DPI, keyboard, accessibility
3. **Phase 3: Integration** — Cross-module coordination

Each phase has a verification gate—a set of commands that must pass before moving to the next phase.

## Phase 1: Foundation Verification Gate

**Acceptance Criteria:**
- Compilation succeeds with no warnings
- All Backend trait methods are present (12 methods)
- Window creation works without crashes
- Basic event pump loop runs

**Test Commands:**
```bash
# Verify compilation
cargo build --target x86_64-unknown-linux-gnu
cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings

# Verify all 12 Backend trait methods exist
grep -n "fn open\|fn pump\|fn surface\|fn appearance\|fn present\|fn is_open\|fn is_fullscreen\|fn set_fullscreen\|fn clipboard_text\|fn set_clipboard_text\|fn set_composition_area\|fn update_accessibility" src/shell/platform/x11.rs

# Format check
cargo fmt --check

# Ensure no regressions in core library
cargo test --lib
```

**Expected Results:**
- ✅ Zero compilation errors
- ✅ Zero clippy warnings
- ✅ All 12 Backend trait methods found in x11.rs
- ✅ Code is properly formatted
- ✅ All library tests pass (379 tests minimum)

## Phase 2: Enhancement Verification Gate

**Acceptance Criteria:**
- DPI detection and scale factor calculation work
- Keyboard event translation is complete
- All X11 event types map to rui Events
- Integration tests pass

**Test Commands:**
```bash
# Full release build
cargo build --release

# X11-specific integration tests
cargo test --test x11_integration

# Full library test suite to ensure no regressions
cargo test --lib
```

**Expected Results:**
- ✅ Release build completes without errors
- ✅ All x11_integration tests pass
- ✅ Full library test suite passes (379+ tests)
- ✅ Event translation verified for:
  - MotionNotify → pointer_moved
  - ButtonPress/Release → pointer pressed/released
  - KeyPress/Release → key events
  - ConfigureNotify → window events

## Phase 3: Integration Verification Gate

**Acceptance Criteria:**
- Platform transparency: identical behavior at any DPI
- Parity with other backends (macOS/Windows)
- Single dispatch path for all input sources
- Cross-module coordination works

**Test Commands:**
```bash
# Cross-platform parity tests
cargo test --test x11_parity

# Interaction tests (pointer and keyboard)
cargo test --test interaction

# All tests pass
cargo test
```

**Expected Results:**
- ✅ x11_parity tests pass (visual consistency with macOS/Windows)
- ✅ interaction tests pass (pointer and keyboard handling)
- ✅ Full test suite passes
- ✅ No platform-specific visual artifacts

## Debugging Checklist

If a gate fails, use this checklist to diagnose:

**Compilation issues:**
- [ ] Run `cargo clean` and rebuild
- [ ] Check for missing X11 headers (libx11-dev, libxcb-dev on Linux)
- [ ] Verify Rust target is correct: `rustc --version --verbose | grep host`
- [ ] Run `cargo check` for detailed error messages

**Event translation issues:**
- [ ] Add debug output in x11.rs event handling code
- [ ] Run `cargo test --test x11_integration -- --nocapture` for verbose output
- [ ] Check event constants in X11 headers match hardcoded values
- [ ] Verify modifier mask calculations for shift/control/alt

**DPI/scale issues:**
- [ ] Check `XrandrGetScreenResourcesCurrent()` output
- [ ] Verify scale factor is clamped to [1.0, 4.0]
- [ ] Test with `RUST_LOG=debug` for detailed scale factor logging
- [ ] Run `xdpyinfo` on the test system to see actual DPI

**Parity issues:**
- [ ] Render same content on macOS, Windows, and X11
- [ ] Compare pixel-exact output with `cargo run -p rui --example gallery`
- [ ] Check focus ring rendering (should be identical)
- [ ] Verify text baseline alignment across platforms

## Commit SHAs and Line Counts

Phase line counts verified against git history:

- **Phase 1**: a67d578eea41560c26fd7a6548c0d089223f3d70 — 748 lines
- **Phase 2**: c42c0f05b3d75976665377a16257c36c472debc1 — 1220 lines
- **Phase 3**: 80e3003563c26952e4d63c52d8eb8f5052cb463c — 1321 lines
- **Polish**: 991167a3898d643199a6e0b9dfa461be31cae264 — 1368 lines
