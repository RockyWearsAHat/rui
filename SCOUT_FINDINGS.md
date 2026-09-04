# Scout Findings — rui Project (2026-09-04)

## Understanding Section (Required)

**rui** is a declarative Rust UI library with zero external dependencies that compiles to native platforms (macOS via Cocoa, Windows via WinAPI, Linux via X11/Wayland) and WebAssembly. The architecture is platform-agnostic: a unified event loop drives five platform backends via a Backend trait, and all rendering flows through a Painter API that automatically respects light/dark themes.

Build with `cargo build`, test with `cargo test`, run examples via `cargo run -p rui --example <name>`. The charter is to ship a production-ready UI toolkit built entirely from primitives, proven by pixel-perfect parity tests across all backends and media query verification for appearance correctness.

Key concepts: state → view function (pure, deterministic, rebuilds each frame) → element tree → layout engine → paint operations → Backend (platform-specific window/rendering) → screen. No interior mutability, no retained widget tree—the view describes the full UI from state, handlers receive mutable state as arguments, and all platform-specific code is isolated to six Backend trait methods (open, pump, surface, appearance, present, is_open).

---

## Proposed Worklist Items (3-5 priority items)

### [scout] [ask: charter-rui] Update CLAUDE.md — Wayland backend completed, remove "planned for v0.2.0"

**Status:** Verification ready
**Why:** CLAUDE.md line 23 states "Wayland support is planned for v0.2.0" but the backend is fully implemented (src/shell/platform/wayland.rs, 18KB, with event handling, buffer management, appearance detection). This misleads developers about project completion status.

**Verification:** 
- `grep -c "planned for v0.2.0" CLAUDE.md` shows 1-2 matches → remove them
- Confirm `ls src/shell/platform/wayland.rs` exists
- Update module structure table in CLAUDE.md to list "Wayland (Wayland protocol)"
- `git diff CLAUDE.md` shows Wayland references updated to past tense

---

### [scout] [ask: charter-rui] Update CLAUDE.md — Accessibility fields implemented, move from future work to complete

**Status:** Verification ready  
**Why:** CLAUDE.md "Accessibility Framework" section exists and describes accessible_name, accessible_role, accessible_description fields on El<S>. However, documentation also states "Platform backend implementation (future work):" suggesting accessibility is not done. The fields exist but platform-specific export to NSAccessibility, UIA, ATK, ARIA is future work. Documentation needs clarification.

**Verification:**
- Confirm accessible_* fields exist in src/element.rs
- Update CLAUDE.md structure to distinguish between "API complete" vs "platform export future work"  
- Accessibility section updated with status: "API complete, platform export future (step 2 of 2)"

---

### [scout] [ask: charter-rui] Add Recipe 3: Form-Building Patterns (text_input, select, combobox)

**Status:** Design + implementation  
**Why:** CLAUDE.md includes comprehensive form control guidance (text_input, select, combobox implementations) but no Recipe document following the Recipe 1/2 structure (commits, phases, files touched, verification gates). Recipe 3 bridges the gap between exemplars (segmented, meter) and production form building, proving the pattern works end-to-end with state management + validation.

**Verification:**
- Recipe 3 block in index.dx with commits, file list, phases, 3+ verification gates
- `cargo test --test recipes -- text_input` passes
- `cargo test --test recipes -- select` passes  
- Commit message: "docs: Add Recipe 3 — Form-Building Patterns (text_input, select, combobox)"

---

### [scout] [ask: charter-rui] Audit and document examples/ — clarify purpose and learning path

**Status:** Documentation  
**Why:** 12 examples exist (counter, controls, calculator, form_example, etc.) but only high-level purpose is in CLAUDE.md. Each example should have inline comments explaining what it teaches and where it fits in the learning path. New contributors should land in examples/ and understand the progression.

**Verification:**
- Each .rs file has a doc comment block explaining its purpose
- README.md examples section lists all 12 with learning path order
- `cargo run -p rui --example <each> --release` succeeds
- Examples follow "copy and modify" template pattern

---

### [scout] [ask: charter-rui] Performance baseline — profile rendering on counter example (all backends)

**Status:** Measurement + documentation  
**Why:** Project claims production-ready UI toolkit but provides no performance benchmarks. Frame time, memory usage, and rendering latency should be baselined on each backend (macOS/Windows/X11/WASM) to enable future optimization and regression detection.

**Verification:**
- Counter renders at 60 FPS, frame time <16ms (native)
- WASM counter animates @ 60 FPS in Firefox, memory stable < 50MB
- Baseline metrics documented in dev.dx or new performance.dx block
- Commit: "perf: Add rendering performance baseline for counter example (all backends)"

---

## Notes

- **Understanding section:** Cannot add to index.dx due to dx schema v5 issue. Recommend upgrading dx or manual fix after tools update.
- **Wayland maturity:** Backend complete but lacks optimization review. Consider "perf: Wayland backend profiling" as follow-up.
- **Accessibility:** API fields exist; platform export (NSAccessibility, UIA, ATK, ARIA) is future work.

