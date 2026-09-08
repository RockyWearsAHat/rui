# Step Decomposition: WASM Capability Gates (bulleted-list-7)

**Original Failed Item:**
Fix 4 failing WASM capability gates in the rui project: cap-docs, cap-wasm-idle-frames, cap-wasm-present-damage, cap-wasm-frame-budget. Each gate requires code additions and tests to advance past current failures.

**Status:** Team dispatch failed (team-general-0 already dispatched). Rewrite into 4 mechanical sub-items ≤10 min each.

**Evidence of Current Failures:**
- `cap-docs`: RUSTDOCFLAGS="-D warnings" cargo doc fails with "unresolved link to `run_wasm`"
- `cap-wasm-idle-frames`: Checking src/shell/mod.rs fails: "exports no frames_drawn() counter"
- `cap-wasm-present-damage`: Checking src/shell/platform/wasm.rs fails: "present() still puts the whole canvas (no dirty rect)"
- `cap-wasm-frame-budget`: Checking tests/ fails: "no tests/frame_budget.rs"

---

## Sub-Item 1: Fix Rustdoc Link in src/shell/mod.rs
- **Task:** In `src/shell/mod.rs` line ~745, find the text `/// A no-op anywhere the program has not called [`run_wasm`]` and change `[run_wasm]` to `[`run_wasm`]` (escape brackets by wrapping in backticks). This fixes the broken intra-doc link reference.
- **Files modified:** src/shell/mod.rs (1 line change)
- **Verification command:** `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`
- **Expected outcome:** Documentation builds without "unresolved link to `run_wasm`" error
- **Capability gate advanced:** [cap: cap-docs]

## Sub-Item 2: Add frames_drawn() Counter for WASM Frame Gating
- **Task:** 
  1. Add `pub fn frames_drawn() -> u32` function to `src/shell/mod.rs` that returns frame draw counter (static u32, incremented in draw loop)
  2. Export `pub fn rui_frames_drawn() -> u32` wrapper in `src/wasm.rs` for JavaScript access
  3. Create `tests/wasm_turn.rs` with 4 tests: 
     - `idle_turn_draws_nothing` — verify frames_drawn stays 0 when no events/animations
     - `event_turn_draws` — verify frames_drawn increments on event arrival
     - `request_redraw_turn_draws` — verify frames_drawn increments on Redraw request
     - `idle_timeout_turn_draws` — verify frames_drawn increments on animation timeout
- **Files modified/created:** src/shell/mod.rs (frames_drawn function), src/wasm.rs (rui_frames_drawn export), tests/wasm_turn.rs (new, 4 tests)
- **Verification command:** `cargo test --test wasm_turn`
- **Expected outcome:** All 4 tests pass; frames_drawn counter tracks draw invocations correctly
- **Capability gate advanced:** [cap: cap-wasm-idle-frames]

## Sub-Item 3: Implement Dirty Rectangle Optimization for WASM Canvas Presentation
- **Task:**
  1. In `src/shell/platform/wasm.rs`, modify `present()` method to track dirty rectangles (bounding box of changed pixels since last frame)
  2. Replace blanket `put_image_data()` call with `put_image_data_with_dirty_rect()` that only updates changed region
  3. Create `tests/present_damage.rs` with 3 tests:
     - `unchanged_frame_presents_nothing` — verify dirty rect is None when frame pixels don't change
     - `one_pixel_change_presents_one_row` — verify dirty rect bounds exactly one changed pixel's row
     - `present_reuses_its_buffer` — verify buffer is recycled across frames, only damage region re-rendered
- **Files modified/created:** src/shell/platform/wasm.rs (dirty rect logic in present), tests/present_damage.rs (new, 3 tests)
- **Verification command:** `cargo test --test present_damage`
- **Expected outcome:** All 3 tests pass; Canvas rendering only updates changed pixels, reducing bandwidth
- **Capability gate advanced:** [cap: cap-wasm-present-damage]

## Sub-Item 4: Create Frame Budget Timing Test
- **Task:**
  1. Create `tests/frame_budget.rs` with 2 frame timing tests:
     - `two_hundred_row_table_frame_under_8ms` — build 200-row table widget, measure single frame time, assert <8ms
     - `twelve_row_frame_under_2ms` — build 12-row table widget, measure single frame time, assert <2ms
  2. Use synthetic fonts and Harness to render deterministically
  3. Report frame timing via println!("frame: {:.2} ms", elapsed_ms) for diagnostics
- **Files created:** tests/frame_budget.rs (new, 2 tests with timing assertions)
- **Verification command:** `cargo test --release --test frame_budget -- --nocapture`
- **Expected outcome:** Both tests pass; frame time under budget in release mode; timing output visible
- **Capability gate advanced:** [cap: cap-wasm-frame-budget]

---

## Sub-Item Dependencies

- **No dependencies between sub-items.** All 4 sub-items are independent:
  - Sub-Item 1 (cap-docs) only touches src/shell/mod.rs docs
  - Sub-Item 2 (cap-wasm-idle-frames) adds new code to src/shell/mod.rs + wasm.rs + tests
  - Sub-Item 3 (cap-wasm-present-damage) modifies src/shell/platform/wasm.rs + tests
  - Sub-Item 4 (cap-wasm-frame-budget) creates new test file only
  - No file conflicts; no order dependencies

## Verification of Decomposition

Each sub-item:
- ✅ Names exact files and commands
- ✅ Has single verifiable outcome (gate passes)
- ✅ Is mechanical (no creative design needed)
- ✅ Takes ≤10 minutes for experienced Rust developer
- ✅ Requires no human submission, signing, or external account

---

## What This Enables After Completion

Once all 4 sub-items pass:
- `cargo doc --no-deps` succeeds (cap-docs)
- `src/shell/mod.rs` exports frames_drawn counter (cap-wasm-idle-frames)
- `src/shell/platform/wasm.rs` optimizes presentation with dirty rects (cap-wasm-present-damage)
- Frame timing budget verified via tests (cap-wasm-frame-budget)

All 4 failing gates advance to "PASS" status, enabling WASM backend to be complete and ready for deployment.
