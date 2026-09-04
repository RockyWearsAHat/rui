# Scout Report: Next Worklist Items for rui

**Scout Run Date:** 2026-08-28
**Status:** Ready to append to index.dx block bulleted-list-7

## Analysis Summary

Recipe 1 (Adding a WASM Backend) is complete and fully documented in CLAUDE.md with all 40 verification checks passing.

The project has established a strong pattern for documenting major features as recipes:
- Clear phase-by-phase structure
- Referenced commits in git history
- Verification gates with exact commands to run
- Cross-module coordination explained

However, three significant features lack recipe documentation:

1. **X11 Backend** - Exists and works, but not documented as a recipe
2. **Custom Controls (Checkbox, etc.)** - Exist in tests/recipes.rs, but not documented as recipes in CLAUDE.md
3. **Recipe Verification Framework** - Need to extend verify_recipes.sh for multiple recipes

## Worklist Items to Append

The following three items should be appended to index.dx block bulleted-list-7 as a checklist:

### Item 1: Document Recipe 2: X11 Backend
```
- [ ] Document Recipe 2: X11 Backend — Reverse-engineer the commits that implemented the X11 backend from git history, document them in CLAUDE.md using the same structure as Recipe 1 (phases, file sequences, verification gates, cross-module coordination). This proves the "Template for the Next Backend" is correct and actionable. Verification: All referenced commits exist in git history, all files mentioned exist, `cargo build` and `cargo test --lib` pass.
```

### Item 2: Document Recipe 3: Custom Control (Checkbox)
```
- [ ] Document Recipe 3: Custom Control (Checkbox) — Document the checkbox implementation from tests/recipes.rs as a recipe in CLAUDE.md with the same structure as Recipe 1 (phases, file sequences, verification gates). This establishes that recipes work for non-backend features. Verification: Recipe structure matches Recipe 1, `cargo test --test recipes -- --nocapture` passes.
```

### Item 3: Extend Recipe Verification Tooling
```
- [ ] Extend recipe verification tooling — Update verify_recipes.sh to check Recipe 2 and Recipe 3 when they're documented. Add checks for control recipes alongside backend recipes. Verification: Script runs without errors, all checks pass for each documented recipe.
```

## How to Apply

Run one of the following:

**Using dx CLI (if MCP integration available):**
```bash
/Users/alexwaldmann/.local/bin/dx append --path index.dx --block bulleted-list-7 --text "- [ ] Document Recipe 2: X11 Backend ..."
# (repeat for each item)
```

**Using the dx MCP append tool:**
```
mcp__dx__dx_append(
  path="index.dx",
  block="bulleted-list-7",
  text="- [ ] Document Recipe 2: X11 Backend ... [full text above]"
)
# (repeat for each item)
```

## Verification

All items include specific verification criteria so the assigned agent knows exactly what "done" means:

- Recipe 2: Git commits exist, files exist, cargo build/test pass
- Recipe 3: Structure matches Recipe 1, tests pass
- Tooling: verify_recipes.sh runs without errors

## Git Commit

Scout analysis committed as: `d252f4a Scout: Identify next work items for worklist`

The scout's decision-making process and detailed reasoning are captured in scout_items_to_append.txt
