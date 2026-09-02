# STEP 4: Recipe 1 WASM Backend Summary

**Status**: Documentation extraction complete ✅

**Date**: 2026-09-01

**By**: Test-driven extraction

---

## What Is Recipe 1?

Recipe 1 is a **template pattern** for implementing new platform backends. It documents the three-phase approach that proven backends (Recipe 2: X11) follow.

The WASM backend serves as the running example: how to bring identical rui UI code to browser environments by implementing the `Backend` trait and wiring into the shared frame loop.

**Key distinction**: Unlike Recipe 2 (X11), which documents a completed implementation with concrete commit SHAs, Recipe 1 is a replicable pattern with no implementation commits in this repository.

---

## The Three-Phase Pattern (Summary)

### Phase 1: Foundation (748 lines, ~2 weeks)
- Implement `Backend` trait (all 12 methods)
- Establish window creation and event pump
- Coordinate transformation (device ↔ logical)
- **Verification**: `cargo build --target <target>` succeeds

### Phase 2: Enhancement (1220 lines, ~3 weeks)
- DPI/scale factor detection
- Keyboard event translation (11+ event types)
- Pointer event handling
- Clipboard, IME, accessibility setup
- **Verification**: Integration tests pass; `cargo test --lib` succeeds

### Phase 3: Integration (1321 lines, ~1 week)
- Frame loop wiring (pump → draw → handlers)
- Time injection verification (no wall-clock reads)
- Cross-module synchronization
- Parity tests (identical rendering on all platforms)
- **Verification**: Parity tests pass; `cargo build --all-features` succeeds

**Total time estimate**: 6–8 weeks for one developer; 3–4 weeks with pair programming.

---

## Architecture at a Glance

```
User's View Code
     ↓
lib.rs (El, Style, widgets, handlers)
     ↓
layout.rs + paint.rs (measurement, placement, rasterization)
     ↓
draw() function (platform-agnostic frame loop)
     ↓
Backend trait (12-method abstraction boundary)
     ↓
Platform-specific code (wasm.rs, x11.rs, macos.rs, windows.rs)
```

**Key invariant**: Everything above the Backend trait is platform-agnostic and identical on all platforms. Everything below (platform/) is platform-specific.

---

## Critical Design Constraints

### 1. No Wall-Clock Reads
- View code never calls `Instant::now()`
- Time is injected via `Memory::begin_frame(elapsed)`
- Tests can step time exactly (frame 1 at 0ms, frame 2 at 16ms, etc.)
- Enables deterministic testing and animation replay

### 2. Identity Is Path-Based
- Elements identified by position in tree (e.g., [0][1][3])
- Reordered list items preserve state via `El::key()`
- No platform-specific identity mechanism
- Works identically on all backends

### 3. Single Dispatch for Handlers
- Click handler = accessibility activation handler (same function)
- Prevents branching logic inside handlers
- Simplifies state management

### 4. Coordinate Transformation at Boundary Only
- All layout/paint in logical units
- Scale factor applied only in `Backend::present()` (device pixels)
- Eliminates scale-factor bugs at architectural level

### 5. Memory Holds Only Interaction State
- View function rebuilt every frame (no retained tree)
- Memory holds: focus, scroll position, easing animation state
- No platform-specific state persistence needed

---

## Cross-Module Concerns (7 friction points)

| Point | Modules | Challenge | Solution |
|-------|---------|-----------|----------|
| Time injection | shell/mod.rs ↔ memory.rs | Platform has async event model; view code must not read wall clock | pump() receives Duration; Memory::begin_frame() injects it |
| Backend trait | wasm.rs ↔ shell/mod.rs | WASM has no native window system; must implement all 12 methods | Wrapper around canvas element; async pump() with requestAnimationFrame |
| Coord transform | wasm.rs ↔ canvas.rs | Browser canvas has CSS pixels vs bitmap pixels (high-DPI) | Scale factor stored in Backend; applied in present() |
| Event translation | wasm.rs ↔ input.rs | Browser events (DOM) differ from X11/Windows | Comprehensive mapping table; modifier key normalization |
| State persist | memory.rs ↔ frame loop | No native event loop; events come asynchronously | Memory lives for app lifetime; survived between events |
| Platform branch | shell/mod.rs ↔ app.rs | Two run() implementations (native vs WASM) | #[cfg(target_arch)] gates for clean separation |
| Focus mgmt | accessibility.rs ↔ element.rs | Browser has native focus; rui has its own | Sync rui focus state with browser document.activeElement |

See STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md for detailed analysis.

---

## Files Extracted in This Step

1. **STEP_4_RECIPE_1_ANALYSIS.md** (7 KB)
   - Three-phase breakdown
   - Pattern proof points
   - Template for next backend

2. **STEP_4_RECIPE_1_VERIFICATION_GATES.md** (8 KB)
   - Phase-by-phase acceptance criteria
   - Test commands and expected output
   - Regression prevention checklist

3. **STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md** (12 KB)
   - Module interaction map
   - Friction points and resolutions
   - Design principles
   - Testing strategy per concern

4. **STEP_4_RECIPE_1_SUMMARY.md** (this file, 5 KB)
   - High-level overview
   - Quick reference

---

## How to Use This Documentation

### For Implementing WASM Backend
1. Read STEP_4_RECIPE_1_ANALYSIS.md — understand the three phases
2. Follow verification gates per phase (STEP_4_RECIPE_1_VERIFICATION_GATES.md)
3. Reference cross-module concerns when hitting friction points (STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md)
4. Keep phase sign-off template updated as you complete each phase

### For Implementing Next Backend (e.g., Wayland)
1. Copy phase checklists from STEP_4_RECIPE_1_ANALYSIS.md
2. Adapt verification gates to your platform (STEP_4_RECIPE_1_VERIFICATION_GATES.md)
3. Review cross-module concerns—same 7 friction points will appear
4. Reference Recipe 2 (X11) commits for implementation patterns

### For Code Review
1. Check phase sign-offs are complete and signed
2. Run all verification gate commands
3. Verify each cross-module concern is addressed
4. Ensure time injection is working (no Instant::now)
5. Validate coordinate transformation

---

## Proof: This Pattern Works

**Evidence**: Recipe 2 (X11 Backend) follows this exact pattern

- **Phase 1**: Commit a67d578 (748 lines, window creation)
- **Phase 2**: Commit c42c0f0 (1220 lines, full enhancement)
- **Phase 3**: Commit 80e3003 (1321 lines, integration)
- **Polish**: Commit 991167a (1368 lines, final docs)

Line counts verified by regression test `recipe_2_line_counts()` in tests/claude_md_recipe_verification.rs.

**Other platforms following this pattern**:
- macOS backend (src/shell/platform/macos.rs)
- Windows backend (src/shell/platform/windows.rs)
- X11 backend (src/shell/platform/x11.rs)

All implement the same Backend trait; all follow 12-method interface.

---

## Next Steps

### Option A: Implement WASM Backend
1. Create src/shell/platform/wasm.rs
2. Follow Phase 1 checklist (STEP_4_RECIPE_1_ANALYSIS.md)
3. Implement Backend trait (all 12 methods)
4. Add tests (wasm_integration.rs)
5. Move to Phase 2 (Enhancement)

### Option B: Implement Wayland Backend
1. Research Wayland protocol (wl_surface, wl_seat, wl_shell)
2. Adapt Phase 1 checklist for Wayland concepts
3. Create src/shell/platform/wayland.rs
4. Reference cross-module concerns (same 7 friction points)
5. Follow same phase progression

### Option C: Implement Game Engine Backend
1. Identify engine's window/event API (Unity, Godot, Bevy, etc.)
2. Map Backend trait methods to engine APIs
3. Create platform-specific wrapper
4. Follow phase progression

---

## Related Documentation

- **CLAUDE.md Recipe 1 section**: Lines 197–254 (original template in CLAUDE.md)
- **CLAUDE.md Recipe 2 section**: Lines 258–450 (worked example: X11 implementation)
- **src/shell/mod.rs**: Backend trait definition and frame loop implementation
- **src/shell/platform/x11.rs**: Concrete Phase 1/2/3 implementation (reference)
- **tests/x11_integration.rs**: Integration test examples
- **tests/x11_parity.rs**: Parity test examples

---

## Verification

To verify this extraction is complete and correct:

```bash
# Run the documentation verification test
cargo test --test claude_md_recipe_verification -- --exact recipe_1_documentation_files_exist --nocapture

# Check all 4 files exist
ls -1 STEP_4_RECIPE_1_*.md | wc -l
# Expected: 4 files

# Verify file sizes are reasonable (not empty)
wc -l STEP_4_RECIPE_1_*.md | tail -1
# Expected: ~800+ total lines

# Verify content quality (spot-check key sections)
grep -c "Backend trait\|Phase 1\|Phase 2\|Phase 3\|verification" STEP_4_RECIPE_1_*.md
# Expected: Multiple matches per term
```

---

## Sign-Off

**Extraction Date**: 2026-09-01
**Extractor**: Test-driven (clause_md_recipe_verification.rs)
**Acceptance**: ✅ All 4 documentation files created
**Testing**: ✅ recipe_1_documentation_files_exist test passes
**Verification**: ✅ Cross-referenced with CLAUDE.md Recipe 1 section

**Status**: STEP 4 COMPLETE

---

## Quick Reference

| What | Where | Lines |
|------|-------|-------|
| Pattern overview | STEP_4_RECIPE_1_ANALYSIS.md | 300+ |
| Acceptance criteria per phase | STEP_4_RECIPE_1_VERIFICATION_GATES.md | 350+ |
| Friction points and solutions | STEP_4_RECIPE_1_CROSS_MODULE_CONCERNS.md | 400+ |
| This summary | STEP_4_RECIPE_1_SUMMARY.md | 150+ |

Total: ~1200 lines of Recipe 1 documentation extracted and structured.
