# STEP 4: Recipe 1 WASM Backend Verification Gates

**Purpose**: Checklist and test commands for verifying Recipe 1 pattern implementation at each phase

**Note**: These are template gates. Since Recipe 1 is a pattern document with no concrete implementation, these gates describe what a WASM backend implementation would verify.

---

## Phase 1: Foundation Verification

### Goal
Backend trait implemented; window creation and basic event pump working.

### Acceptance Criteria

**Compilation succeeds**:
```bash
cargo build --target wasm32-unknown-unknown
```
Expected: No errors, no warnings.

**All 12 Backend trait methods present**:
```bash
grep -n "fn open\|fn pump\|fn surface\|fn appearance\|fn present\|fn is_open\|fn is_fullscreen\|fn set_fullscreen\|fn clipboard_text\|fn set_clipboard_text\|fn set_composition_area\|fn update_accessibility" src/shell/platform/wasm.rs
```
Expected: Exactly 12 methods found and implemented (not panicking stubs).

**Code formatting**:
```bash
cargo fmt --check
```
Expected: No formatting errors.

**No clippy warnings**:
```bash
cargo clippy --target wasm32-unknown-unknown -- -D warnings
```
Expected: No warnings.

**Library tests unchanged**:
```bash
cargo test --lib
```
Expected: Same number of passing tests (379+) as before the implementation.

### Success Criteria
- ✅ Compilation succeeds for wasm32-unknown-unknown target
- ✅ All 12 Backend trait methods implemented
- ✅ Code formatted and clippy-clean
- ✅ No test regressions in library

---

## Phase 2: Enhancement Verification

### Goal
Full feature parity: DPI detection, keyboard translation, pointer events, clipboard, IME, accessibility.

### Acceptance Criteria

**Release build succeeds**:
```bash
cargo build --release --target wasm32-unknown-unknown
```
Expected: No errors or warnings.

**Integration tests pass**:
```bash
cargo test --test wasm_integration --nocapture
```
Expected: All tests pass; output shows:
- Pointer event translation (motion, press, release)
- Keyboard event translation (key codes → rui Key)
- Modifier key handling (shift/control/alt)
- Scale factor detection (1.0 for browser canvas)
- Clipboard read/write (if supported by browser)

**Full library test suite still passes**:
```bash
cargo test --lib
```
Expected: 379+ tests pass; no new failures.

**File size reasonable**:
```bash
wc -l src/shell/platform/wasm.rs
```
Expected: Between 500–1500 lines (similar scope to phase-2 platforms).

### Success Criteria
- ✅ Integration tests pass
- ✅ Pointer and keyboard events working
- ✅ Scale factor handled (1.0 for standard canvas)
- ✅ Library tests still pass
- ✅ File size ~700–1000 lines

---

## Phase 3: Integration Verification

### Goal
Frame loop wired; parity with native backends; identical visual output.

### Acceptance Criteria

**Platform branching added**:
```bash
grep -n "cfg(target_arch = \"wasm32\")" src/shell/mod.rs src/app.rs
```
Expected: Feature gates in place for wasm32-specific code paths.

**Parity tests pass**:
```bash
cargo test --test wasm_parity --nocapture
```
Expected: Tests verify:
- Same frame rendered at same resolution looks identical to X11/macOS/Windows
- Keyboard input translated identically across platforms
- Pointer events report same coordinates
- Focus management works identically

**All features build**:
```bash
cargo build --all-features --target wasm32-unknown-unknown
```
Expected: No errors; all platform features gated correctly.

**Time injection verified** (no wall-clock reads):
```bash
grep "std::time::Instant::now" src/shell/platform/wasm.rs
```
Expected: No matches (time is injected via `Memory::begin_frame()`).

**Code review checklist**:
- [ ] `pump()` receives injected `Duration`, doesn't call `Instant::now()`
- [ ] Event translation complete (11+ wasm event types → rui Events)
- [ ] Coordinate transformation correct (canvas pixels → logical units)
- [ ] Modifier key bits normalized (shift/control/alt/meta)
- [ ] Focus management uses `El::takes_focus` consistently
- [ ] Accessibility tree updates per `AccessUpdate`
- [ ] Clipboard operations gated for browser APIs that support them

### Success Criteria
- ✅ Platform-agnostic code above Backend trait boundary
- ✅ Parity tests pass (visual output identical)
- ✅ Time injection verified (no Instant::now)
- ✅ All features compile
- ✅ Code review checklist passed

---

## Cross-Module Verification

### Time Injection (src/shell/mod.rs + src/memory.rs)
**Test**: Frame loop provides elapsed time; view code never reads clock
```bash
# Verify: no Instant::now() in non-platform code
grep -r "Instant::now" src/ | grep -v "shell/platform/" | wc -l
```
Expected: 0 matches (time confinement verified).

### Backend Trait (src/shell/mod.rs)
**Test**: All backends implement identical trait
```bash
# Count methods in Backend trait
grep -A 100 "pub trait Backend" src/shell/mod.rs | grep "fn " | wc -l
```
Expected: Exactly 12 methods.

### Event Translation (src/input.rs)
**Test**: Platform events convert to platform-agnostic Input
```bash
# Verify Input type exists and is used universally
grep -n "struct Input\|fn to_input\|Event → Input" src/input.rs | head -5
```
Expected: Input type and translation functions present.

### Focus Consistency (src/accessibility.rs)
**Test**: `El::takes_focus` is the single source
```bash
grep -n "takes_focus\|focusable.*disabled" src/accessibility.rs | head -3
```
Expected: Focus checks unified through `takes_focus()`.

---

## Testing Strategy

### Unit Tests
- Event translation (wasm event → rui Event)
- Coordinate transformation (canvas → logical)
- Modifier key normalization

### Integration Tests (tests/wasm_integration.rs)
- Pointer events (motion, click, drag)
- Keyboard events (letters, navigation, modifiers)
- Focus management (Tab navigation)
- Clipboard (if browser API available)

### Parity Tests (tests/wasm_parity.rs)
- Same UI rendered at 1.0 DPI produces identical pixels to X11/macOS/Windows at 1.0 DPI
- Keyboard input produces same handlers across platforms
- Tab order identical across platforms

### Visual Regression Tests
- Golden frame images stored for key screens
- Rendered frame compared to golden (pixel-perfect or tolerance-based)
- Diff reports any visual changes

---

## Regression Prevention

### Invariants to Preserve
1. **View rebuilt every frame** — No retained state in view code
2. **Time injected, never read** — `Instant::now()` forbidden outside platform/
3. **Identity is path-based** — Reordered list items follow position, not content
4. **Event translation complete** — Every wasm event type has rui equivalent
5. **Coordinate transformation** — Device pixels ↔ logical units consistently
6. **Focus single-source** — `El::takes_focus` is only focusability check
7. **Scale factor range** — Validated (1.0–4.0 typical)

### Test Additions
- `test_wasm_time_injection()` — Verify time injected, never read
- `test_event_translation_complete()` — Verify all event types handled
- `test_coordinate_transformation()` — Verify canvas→logical correct
- `test_parity_with_x11()` — Verify identical frame rendering

---

## Sign-off Template

```
Phase 1 Foundation:
✅ Compilation succeeds (wasm32-unknown-unknown)
✅ All 12 Backend trait methods implemented
✅ No clippy warnings
✅ Library tests unchanged
Verified by: [name] on [date]

Phase 2 Enhancement:
✅ Integration tests pass (pointer, keyboard, modifiers)
✅ Scale factor detected/validated
✅ Clipboard operations working (where supported)
✅ Library tests still pass (379+ tests)
Verified by: [name] on [date]

Phase 3 Integration:
✅ Platform branching in shell/mod.rs and app.rs
✅ Parity tests pass (pixel-perfect frames)
✅ Time injection verified (no Instant::now)
✅ All features compile (--all-features)
Verified by: [name] on [date]

Overall Sign-off: WASM backend ready for production
Verified by: [name] on [date]
```

---

## References

- **CLAUDE.md Recipe 1**: Lines 197–254
- **Backend trait**: src/shell/mod.rs line 183
- **Frame loop**: src/shell/mod.rs line 305+
- **Event types**: src/input.rs
- **Memory/state**: src/memory.rs
- **Recipe 2 tests** (reference): tests/x11_integration.rs, tests/x11_parity.rs
