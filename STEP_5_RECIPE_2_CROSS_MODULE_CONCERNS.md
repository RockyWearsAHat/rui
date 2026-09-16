# Recipe 2: X11 Backend — Cross-Module Concerns

Reference document for Recipe 2 (X11 Backend Implementation). This file identifies where the X11 platform backend interacts with other modules and how to resolve friction points.

## Module Interaction Map

The X11 backend sits at the platform boundary, translating between X11 protocol and rui's platform-agnostic abstractions:

```
X11 Protocol
    ↓
src/shell/platform/x11.rs  ← implements Backend trait
    ↓ pump() returns Vec<Event>
src/shell/mod.rs  ← platform selector, generic event dispatch
    ↓
src/app.rs  ← frame loop calls Backend methods
    ↓ pump() events
src/input.rs  ← Event → Input translation
    ↓
src/paint.rs  ← Pointer locations, key presses
    ↓
src/memory.rs  ← Focus, scroll, interaction state
    ↓
src/accessibility.rs  ← Tree queries for a11y
```

## Key Interactions

### 1. Backend Trait Boundary (app.rs ↔ x11.rs)

**What happens:**
- `app.rs` owns the frame loop
- Calls `Backend::pump()` to get events
- Calls `Backend::present()` to render to screen
- Calls `Backend::surface()` for window dimensions and scale factor

**X11-specific responsibility:**
- Pump returns `Vec<Event>` from X11 event queue
- Present writes canvas buffer to X11 drawable via XPutImage
- Surface returns (width, height, scale_factor) from window and DPI

**Friction points:**
- X11 events are raw protocol messages; rui Events are platform-agnostic
- Mismatch: does ButtonPress mean left or right button? X11 uses BTN_LEFT (1), BTN_MIDDLE (2), BTN_RIGHT (3)
- Solution: Translate in pump() before returning Vec<Event>

**How to verify:**
- Read app.rs::Backend trait boundary (line 305+)
- Confirm pump() signature returns `Vec<Event>`
- Check x11.rs pump() method matches exactly

### 2. Platform Selection (shell/mod.rs ↔ x11.rs)

**What happens:**
- shell/mod.rs uses `#[cfg(target_os = "linux")]` to select X11 backend
- X11 is default; Wayland is opt-in via `--features wayland`
- Generic `run()` function calls platform-specific implementations

**X11-specific responsibility:**
- Provide platform-specific `run()` function
- Implement Backend trait completely (all 12 methods)

**Friction points:**
- Feature gating can cause compilation breakage if X11 is optional
- Solution: Keep X11 as default; Wayland is `--features wayland`

**How to verify:**
- Check src/shell/mod.rs for `#[cfg(target_os = "linux")]`
- Verify platform selector matches target OS
- Run `cargo build --all-features` on multiple platforms

### 3. Event Translation (x11.rs → input.rs → paint.rs)

**What happens:**
1. X11 delivers events (MotionNotify, ButtonPress, KeyPress, etc.)
2. x11.rs pump() translates to rui Event types
3. input.rs converts Event → Input (resolved key meanings)
4. paint.rs reads Pointing/stroke data when drawing handlers

**X11-specific responsibility:**
- Translate X11 event types to rui Events
- Map X11 KeyCode to rui Key (via XKB layout)
- Handle modifier masks (shift, control, alt)

**Friction points:**
- X11 KeyPress gives KeyCode (position), not keysym (meaning)
- Solution: x11.rs must call XkbLookupKeySym to get meaning
- Mismatch: Some keys have no X11 equivalent (e.g., Cmd key)
- Solution: Translate to Key::Unknown, let input.rs handle it

**Event types to translate:**
- **MotionNotify** → Event::Pointer with moved=true
- **ButtonPress** → Event::Pointer with pressed=true (map button number)
- **ButtonRelease** → Event::Pointer with released=true
- **KeyPress/KeyRelease** → Event::Key (resolved Key, modifiers)
- **ConfigureNotify** → Event::Resized or Event::ScaleFactorChanged
- **FocusIn/FocusOut** → Event::FocusChanged
- **Expose** → implies Redraw needed (handled by pump loop)

**How to verify:**
- Read x11.rs pump() method completely
- Check Event translation tests in tests/x11_integration.rs
- Run examples and verify keyboard/mouse/window events work

### 4. Coordinate Transformation (x11.rs ↔ layout.rs ↔ paint.rs)

**What happens:**
1. X11 delivers pointer events in device pixels
2. x11.rs translates to logical units: `logical = device / scale_factor`
3. layout.rs and paint.rs work entirely in logical units
4. Before presenting, canvas.rs translates back: `device = logical * scale_factor`

**X11-specific responsibility:**
- Detect DPI and compute scale_factor
- Translate incoming pointer events to logical coordinates
- Store scale_factor for use by shell/mod.rs::Backend trait

**Friction points:**
- X11 has no standard DPI query; must use XrandrGetScreenResourcesCurrent
- Mismatch: Some systems report DPI as 96, others as 72, others as 2.0× scale
- Solution: Query Xrandr, fall back to environment if unavailable
- Scale factor must be in [1.0, 4.0]; clamp aggressively

**Contract:**
```
logical_x = device_x / scale_factor
logical_y = device_y / scale_factor
```

**How to verify:**
- Read x11.rs::detect_dpi()
- Check that pointer events use scale_factor correctly
- Test at DPI 96, 192, custom values
- Verify layout at 1.0x and 2.0x scale matches macOS/Windows

### 5. Focus and Keyboard State (memory.rs ↔ x11.rs)

**What happens:**
1. X11 delivers FocusIn/FocusOut events
2. x11.rs translates to Event::FocusChanged
3. memory.rs tracks which element has focus
4. On next frame, paint.rs draws focus ring

**X11-specific responsibility:**
- Map X11 focus events to rui Event::FocusChanged
- Request X11 input method composition area via XSetICFocus

**Friction points:**
- X11 focus is window-level; rui focus is element-level
- Solution: x11.rs reports window focus; paint.rs/memory.rs handle element focus
- IME: X11 InputMethod needs to know where focused text field is
- Solution: x11.rs receives set_composition_area call and forwards to X11

**How to verify:**
- Verify FocusIn/FocusOut events are translated correctly
- Test with Tab key: focus should move between focusable elements
- Test IME: composition area should move with text cursor
- Verify focus ring renders only on keyboard focus (not mouse click)

### 6. Accessibility Tree (accessibility.rs ↔ x11.rs)

**What happens:**
1. accessibility.rs builds semantic tree of elements
2. x11.rs implements update_accessibility() method
3. X11 AT-SPI2 bridge uses tree for screen readers

**X11-specific responsibility:**
- Implement Backend::update_accessibility() to expose tree to X11 a11y
- Register ATContext with AT-SPI2 daemon
- Update node roles, names, states as tree changes

**Friction points:**
- X11 a11y requires AT-SPI2 daemon (not always running)
- Solution: Fail gracefully if daemon unavailable; don't crash
- Mismatch: rui roles don't perfectly match AT-SPI2 roles
- Solution: Map rui Role enum to closest AT-SPI2 role

**How to verify:**
- Run with screen reader (Orca on Linux)
- Check that buttons, fields, labels are announced correctly
- Verify Tab order matches declaration order
- Test with `cargo test --lib accessibility`

## Resolution Patterns

### Pattern 1: Event Translation Mismatch
**Problem:** X11 event format ≠ rui event format
**Solution:** Translate completely in pump(), return rui Events only
**Example:** ButtonPress with button=1 → Event::Pointer(pressed=true, button=Primary)

### Pattern 2: Scale Factor Uncertainty
**Problem:** Multiple ways to query DPI on X11
**Solution:** Try Xrandr first, fall back to environment, clamp to [1.0, 4.0]
**Example:** DPI 192 → scale_factor 2.0; DPI 72 → scale_factor 1.0

### Pattern 3: Key Meaning Lookup
**Problem:** X11 KeyPress gives position (KeyCode), not meaning (Key)
**Solution:** Call XkbLookupKeySym with layout before returning Event::Key
**Example:** KeyCode 38 → (Shift pressed) → Key::A instead of Key::Unknown

### Pattern 4: Graceful Degradation
**Problem:** X11 feature may not be available (IME, a11y, Xrandr)
**Solution:** Check availability, use sensible defaults if missing
**Example:** No Xrandr → assume 1.0 scale; no IME → skip set_composition_area

## Verification Checklist

When implementing X11 backend, check these cross-module interactions:

- [ ] Backend trait: all 12 methods implemented
- [ ] Event translation: MotionNotify, ButtonPress, KeyPress, ConfigureNotify all translate correctly
- [ ] Coordinate transformation: scale_factor applied to pointer events
- [ ] Focus management: FocusIn/Out events translated, composition area set
- [ ] Accessibility: update_accessibility() method exists and doesn't crash
- [ ] Platform selection: src/shell/mod.rs correctly selects X11 for target_os="linux"
- [ ] No compilation warnings with `cargo clippy`
- [ ] All tests pass: `cargo test --lib` (379+ tests)
- [ ] Platform parity: visual output matches macOS/Windows at 1.0x and 2.0x scale
- [ ] Graceful degradation: missing X11 features don't cause panics

## Next Backend Template

When implementing Wayland or another backend, reuse this interaction map:

1. **Backend trait** — Implement all 12 methods exactly
2. **Event translation** — Map protocol events to rui Events in pump()
3. **Coordinate transformation** — Apply scale_factor consistently
4. **Platform selection** — Add `#[cfg(...)]` block in shell/mod.rs
5. **Focus and IME** — Handle focus events, composition area
6. **Accessibility** — Implement update_accessibility() with graceful fallback

The interaction pattern is protocol-agnostic; only the event translation and platform queries change per backend.
