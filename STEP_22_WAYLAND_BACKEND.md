# STEP 22: Wayland Backend Implementation

## Overview

**rui** now supports macOS (Cocoa), Windows (WinAPI), and Linux (X11). This step adds Wayland support, enabling modern Linux distributions (GNOME, KDE Plasma, wlroots) to run rui applications without X11.

Wayland is the next-generation display server protocol, replacing X11 on many Linux systems. By implementing the `Backend` trait for Wayland, we extend **rui** to work seamlessly on both legacy (X11) and modern (Wayland) Linux environments.

## Architecture Pattern

Following the three-phase pattern from Recipe 1 (WASM) and Recipe 2 (X11):

### Phase 1: Foundation (1 commit)
**Objective:** Implement the `Backend` trait for Wayland with minimal functionality.

Files to create/modify:
- `src/shell/platform/wayland.rs` (new): Core Backend trait implementation
  - Wayland protocol FFI bindings via `wayland-client` crate
  - Window creation via `wl_surface` and `xdg_toplevel`
  - Event pump collecting `wl_callback` signals
  - Basic coordinate handling

- `src/shell/mod.rs`: Add conditional compilation for Wayland backend
  - `#[cfg(target_os = "linux")] pub(crate) fn run<S: 'static>()` variant for Wayland
  - Wayland backend selection logic

Verification gate:
```bash
cargo build --target x86_64-unknown-linux-gnu --features wayland
cargo test --lib
```

### Phase 2: Enhancement (1 commit)
**Objective:** Add full feature parity with X11 and WASM backends.

Files to extend:
- `src/shell/platform/wayland.rs`:
  - **DPI Detection:** Query `wl_output` for physical dimensions and current mode
  - **Keyboard Support:** Map `xkb_keysym` to rui's `Key` enum with modifiers
  - **Appearance Detection:** Query `_wp_color_management_v1` or fallback to `gtk-application-prefer-dark-theme`
  - **Mouse Events:** Map `wl_pointer` events (enter, leave, motion, button)
  - **Keyboard Events:** Setup `xkb` library for keymap translation

- `src/text.rs` (if needed): Ensure system font paths work on Wayland (`/usr/share/fonts`)

Verification gates:
```bash
cargo test --test wayland_integration
cargo test --lib geometry           # Layout still works
cargo test --lib paint              # Rendering still works
```

### Phase 3: Integration (8 commits)
**Objective:** Verify cross-platform consistency and establish regression prevention.

Files:
- `tests/wayland_parity.rs`: Pixel-perfect comparison tests
  - Render the same scene on Wayland and native (X11/macOS)
  - Compare PNG output to verify zero differing pixels
  - Test both light and dark modes

- `tests/wayland_integration.rs`: Event handling and state management tests
  - Verify app responds to Wayland pointer events (click, drag)
  - Verify keyboard input and modifiers work correctly
  - Verify appearance detection (light/dark mode switching)
  - Verify coordinate translation at various DPI scales

- `src/shell/mod.rs`: Enhanced validation
  - Verify timeout semantics are consistent with X11
  - Ensure 60 FPS refresh rate achievable on Wayland

Documentation:
- Update `CLAUDE.md` Troubleshooting section:
  - Wayland setup instructions (install `wayland-protocols`, `libxkbcommon`)
  - How to force Wayland vs. X11 (`GDK_BACKEND=wayland`)
  - Common issues and solutions

Verification gates:
```bash
cargo test --test wayland_parity        # Zero differing pixels
cargo test --test wayland_integration   # All events work correctly
cargo test --lib                         # No regressions
```

## Acceptance Criteria

- [ ] Phase 1 Foundation commits compile without warnings
- [ ] Phase 1 Backend trait fully implemented for Wayland
- [ ] Phase 2 Enhancement adds DPI detection, keyboard, appearance support
- [ ] Phase 2 Feature parity verified with X11/WASM backends
- [ ] Phase 3 Wayland integration tests pass (event handling, state persistence)
- [ ] Phase 3 Parity tests show zero differing pixels (light and dark modes)
- [ ] Phase 3 Regression prevention tests pass (all platform backend consistency)
- [ ] `CLAUDE.md` updated with Wayland troubleshooting section
- [ ] All 350+ library tests still passing
- [ ] Pre-commit hook passes (formatting + clippy)
- [ ] Commit history documents each phase clearly

## Deliverables

### Code
- `src/shell/platform/wayland.rs`: ~500-800 lines
  - Wayland FFI bindings and protocol handling
  - Backend trait implementation (6 methods)
  - Coordinate translation and event processing

- Test suite `tests/wayland_integration.rs`: ~300 lines
  - 20+ integration tests covering events, state, appearance

- Parity test `tests/wayland_parity.rs`: ~150 lines
  - Pixel-perfect comparison (light and dark modes)

### Documentation
- `STEP_22_VERIFICATION.txt`: Full verification report with test results
- Updated `CLAUDE.md`: Wayland setup, troubleshooting, platform-specific notes

### Testing
- All phases include verification gates (compile, run, parity)
- Pre-commit hook configured for Wayland development
- Regression prevention tests ensure no breakage on other platforms

## Cross-Module Concerns

### Coordination Points

1. **`shell/clock.rs` ↔ `shell/mod.rs`**
   - Wayland uses `wl_callback` for frame timing (no blocking wait like X11)
   - Time measurement must use `shell::clock::Moment` (already abstracts this)

2. **`Backend` trait ↔ `turn()` function**
   - Wayland implements all 6 trait methods identically to X11/WASM
   - Event collection via `wl_callback` (non-blocking, like WASM)
   - No changes needed above trait boundary

3. **Event Flow**
   - Wayland events: `wl_pointer`, `wl_keyboard`, `wl_surface`
   - Must translate to rui's unified `Event` type (Click, Drag, Key, etc.)
   - Same as X11 path; coordinate translation handles DPI

4. **Appearance Detection**
   - Wayland: Query `wl_output` and desktop portal (`xdg-desktop-portal`)
   - Fallback to `GTK_THEME` environment variable
   - Same path as X11's appearance detection

5. **Platform Isolation**
   - All Wayland-specific code in `src/shell/platform/wayland.rs`
   - No changes to layout, paint, text, or element modules
   - Unsafe code confined to FFI bindings (like X11)

## Implementation Notes

### Wayland Dependencies

The implementation uses:
- `wayland-client`: Official Wayland protocol bindings
- `wayland-protocols`: Extended protocol definitions (xdg-shell, xkb-common)
- `xkbcommon` (via `xkb-sys`): Keyboard layout translation

These should be added to `Cargo.toml` with `#[cfg(target_os = "linux")]` to keep them Linux-only and preserve the "zero dependencies" promise for non-Linux targets.

### Event Loop Model

Unlike X11 (which uses `XNextEvent` blocking), Wayland uses a callback-based event model:
- `wl_callback` is armed after `wl_surface_commit()`
- Callback fires when the display is ready for the next frame
- Multiple events may buffer (pointer, keyboard, etc.) between callbacks
- The same `turn()` loop works; events are collected in the callback handler

### Coordinate System

Wayland reports coordinates in logical pixels (DPI-independent). The X11 backend had to translate device pixels to logical; Wayland does this for us. Verify this with parity tests:
- Wayland click at (100, 100) should trigger same element as X11 click at (100, 100)
- At 2x DPI, Wayland still reports (100, 100) not (200, 200)

## Testing Strategy

### Unit Tests (Phase 1)
```bash
cargo test --lib                                  # All library tests
```

### Integration Tests (Phase 2)
```bash
cargo test --test wayland_integration            # Event handling, state
```

### Parity Tests (Phase 3)
```bash
cargo test --test wayland_parity                 # Pixel-perfect comparison
cargo test --test recipe_1_verification          # Memory persistence
```

### Regression Tests (Phase 3)
```bash
cargo test --lib                                 # All platforms still work
cargo test --test interaction                    # Input handling on all backends
cargo test --test backend_consistency            # Coordinate translation verified
```

## Regression Prevention

After implementing Wayland, run this checklist before merging:

**Immediate (before committing):**
```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test --test wayland_integration
```

**Before merging to main:**
```bash
cargo test --lib                                # All platform tests
cargo test --test wayland_parity                # Parity verification
cargo test --test interaction                   # No event handling regressions
cargo test --test recipe_1_verification         # WASM still works
```

**On CI/CD (Linux runner):**
```bash
cargo build --target x86_64-unknown-linux-gnu --features wayland
cargo test --all-targets
```

## Troubleshooting (To Be Documented)

Common Wayland setup issues:

1. **"Wayland connection failed"**
   - Verify Wayland is running: `echo $WAYLAND_DISPLAY`
   - Should output something like `wayland-0` or `wayland-1`
   - If empty, you're on X11 or Wayland isn't available

2. **"libwayland not found"**
   - Install: `sudo apt-get install libwayland-dev` (Ubuntu/Debian)
   - Or: `sudo yum install wayland-devel` (RHEL/Fedora)

3. **Keyboard input not working**
   - Wayland requires `xkbcommon` for keymap translation
   - Install: `sudo apt-get install libxkbcommon-dev`
   - Verify: `pkg-config --cflags xkbcommon` should succeed

4. **Parity test shows differences**
   - Check DPI scaling: `xrandr | grep -E "connected|primary"`
   - Compare logical pixel sizes between X11 and Wayland
   - Verify coordinate transformation formulas

## Next Steps After Wayland

Once Wayland is complete and verified:

1. **Mobile Support (iOS/Android):** Platform backends for touch-based systems
2. **Web Improvements:** Enhance WASM backend with better touch event support
3. **Advanced Features:** Drag-and-drop, file dialogs, clipboard across platforms
4. **Performance:** Profile on each platform, optimize hot paths

## Sign-Off Template

Once all phases are complete, create `STEP_22_VERIFICATION.txt` with:

```
STEP 22: WAYLAND BACKEND IMPLEMENTATION
========================================

Status: ✅ COMPLETE

Phase 1: Foundation — ✅ VERIFIED
Phase 2: Enhancement — ✅ VERIFIED
Phase 3: Integration — ✅ VERIFIED

Test Results:
- Library tests: 350+ passed, 0 failed
- Wayland integration tests: 20+ passed, 0 failed
- Parity tests: 0 differing pixels (light & dark)
- All platforms: No regressions

Deliverables:
- src/shell/platform/wayland.rs (600+ lines)
- tests/wayland_integration.rs (300+ lines)
- tests/wayland_parity.rs (150+ lines)
- CLAUDE.md updated with Wayland troubleshooting

Status: WAYLAND BACKEND PRODUCTION READY
```

---

## Implementation Checklist

- [ ] Create `src/shell/platform/wayland.rs` with Foundation phase
- [ ] Add Wayland conditional compilation to `src/shell/mod.rs`
- [ ] Implement all 6 Backend trait methods for Wayland
- [ ] Phase 1 verification: compilation + basic trait tests
- [ ] Add DPI detection, keyboard, appearance support (Phase 2)
- [ ] Create integration test suite (Phase 2)
- [ ] Create parity test suite (Phase 3)
- [ ] Verify coordinate contract (Phase 3)
- [ ] Update CLAUDE.md with Wayland section (Phase 3)
- [ ] All tests passing (Phase 3)
- [ ] Pre-commit hook passing (Phase 3)
- [ ] Commit each phase with clear message (Phase 1, 2, 3)
- [ ] Create STEP_22_VERIFICATION.txt (Phase 3)

---

**Start Date:** 2026-08-30
**Target Completion:** TBD
**Status:** Ready to begin Phase 1
