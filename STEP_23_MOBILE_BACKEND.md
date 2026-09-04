# STEP 23: Mobile Backend Implementation (iOS/Android)

## Overview

STEP 23 extends **rui** to mobile platforms (iOS/Android), following the established three-phase Recipe 2 pattern. This step creates native mobile backends that implement the unified `Backend` trait, allowing the same UI code to run on phones and tablets with no changes.

**Key distinction:**
- **Desktop/Web backends** (macOS, Windows, X11, Wayland, WASM): Window-based, pixel-perfect rendering
- **Mobile backends** (iOS, Android): Full-screen, touch-first, performance-critical

Both implement the identical `Backend` trait. Platform selection is determined by `#[cfg(target_os = "...")]` and feature gates.

---

## Architecture: Three Phases

### Phase 1: Foundation — iOS Backend

**Goal:** Implement the Backend trait for iOS. Prove the abstraction works on mobile.

**Files to create:**
- `src/shell/platform/ios.rs` — ~200 lines, implements Backend trait
- Updates to `src/shell/platform/mod.rs` for iOS platform selection
- Updates to `Cargo.toml` for iOS feature gate

**What Phase 1 does:**

```rust
pub struct IosBackend {
    is_open: bool,
    logical_width: u32,
    logical_height: u32,
    scale_factor: f32,
    appearance: Appearance,
}

impl Backend for IosBackend {
    fn open(_options: &WindowOptions) -> Result<Self, Error> {
        // UIWindow creation via Objective-C FFI
    }
    fn pump(&mut self, _timeout: Duration, events: &mut Vec<Event>, ...) {
        // Collect touch events from UIResponder event chain
    }
    fn surface(&self) -> (u32, u32, f32) {
        // Return screen dimensions and scale factor
    }
    fn appearance(&self) -> Appearance {
        // Query UITraitCollection for light/dark mode
    }
    fn present(&self, _canvas: &Canvas) -> Result<(), Error> {
        // Render canvas to CALayer via Metal
    }
    fn is_open(&self) -> bool {
        self.is_open
    }
}
```

**Platform selector** (in `src/shell/platform/mod.rs`):

```rust
#[cfg(target_os = "ios")]
#[path = "ios.rs"]
mod backend;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod backend;
```

**Verification gate:** iOS target compiles, trait contract verified.

---

### Phase 2: Enhancement — Full Mobile Support

**Goal:** Add touch event translation, DPI detection, appearance detection, lifecycle management.

**Files to update:**
- `src/shell/platform/ios.rs` — Extend with Phase 2 features
- `src/shell/platform/android.rs` (new) — Android backend with same pattern
- `src/input.rs` (if needed) — Add touch-specific event types

**What Phase 2 adds:**

**Touch Event Translation:**
```rust
// iOS UITouch handling
impl Backend for IosBackend {
    fn pump(&mut self, timeout, events, redraw) {
        // Translate UITouch.phase to rui Events
        // UITouchPhaseBegan → Click
        // UITouchPhaseMoved → Drag
        // UITouchPhaseEnded → Release
        // Track multi-touch (ignore for Phase 1)
    }
}
```

**DPI Scaling:**
```rust
fn detect_scale_factor() -> f32 {
    // UIScreen.main.nativeScale (iOS)
    // DisplayMetrics.density (Android)
    // Returns device scale: 1x, 2x, 3x
}
```

**Appearance Detection:**
```rust
fn detect_appearance() -> Appearance {
    // iOS: UITraitCollection.userInterfaceStyle
    // Android: Configuration.uiMode & UI_MODE_NIGHT_MASK
    // Check system appearance (light/dark)
}
```

**Lifecycle Management:**
```rust
// iOS app delegate lifecycle:
// applicationDidFinishLaunching → Backend::open()
// applicationWillTerminate → Backend::is_open() false
// applicationWillResignActive → pause rendering
// applicationDidBecomeActive → resume rendering
```

---

### Phase 3: Integration & Verification

**Goal:** Verify mobile backends work correctly, ensure parity with desktop rendering, test on real devices.

**Files to create:**
- `tests/ios_integration.rs` — Functional integration tests
- `tests/ios_parity.rs` — Rendering comparison tests
- `tests/android_integration.rs` — Android equivalent
- `tests/android_parity.rs` — Android rendering tests

**What Phase 3 adds:**

**Touch Event Verification:**
```rust
#[test]
fn ios_touch_event_translates_to_click() {
    // Simulate UITouch → verify Click event
    // Test single tap, multi-tap, long press
}

#[test]
fn android_touch_event_translates_correctly() {
    // Simulate MotionEvent → verify Click/Drag events
    // Test Android-specific gesture handling
}
```

**Appearance Testing:**
```rust
#[test]
fn ios_light_mode_renders_correctly() {
    // Set UIUserInterfaceStyle.light
    // Verify colors render as expected
}

#[test]
fn ios_dark_mode_switches_correctly() {
    // Toggle UIUserInterfaceStyle
    // Verify appearance changes within single frame
}
```

**Performance Testing:**
```rust
#[test]
fn ios_renders_60fps_on_high_dpi_display() {
    // Test with 3x display scale (iPhone Pro)
    // Verify frame time < 16ms
}

#[test]
fn android_renders_60fps_with_variable_refresh_rate() {
    // Test on device with 90Hz, 120Hz screens
    // Verify smooth scrolling regardless of refresh rate
}
```

---

## Platform Selector Configuration

**Location:** `src/shell/platform/mod.rs`

**Selection order** (first match wins):

1. WASM: `#[cfg(target_arch = "wasm32")]` → `wasm.rs`
2. iOS: `#[cfg(target_os = "ios")]` → `ios.rs`
3. Android: `#[cfg(target_os = "android")]` → `android.rs`
4. macOS: `#[cfg(target_os = "macos")]` → `macos.rs`
5. Windows: `#[cfg(target_os = "windows")]` → `windows.rs`
6. Wayland: `#[cfg(all(target_os = "linux", feature = "wayland"))]` → `wayland.rs`
7. X11: `#[cfg(all(unix, not(macos), not(wasm), not(wayland)]` → `x11.rs`
8. Unsupported: fallback error

**How to enable iOS:**

```bash
# Requires: Apple Silicon Mac + Xcode + iOS SDK
cargo build --target aarch64-apple-ios --features ios
cargo build --target x86_64-apple-ios --features ios-simulator
```

**How to enable Android:**

```bash
# Requires: Android NDK + cargo-ndk
cargo build --target aarch64-linux-android --features android
cargo build --target armv7-linux-android --features android
```

---

## Mobile-Specific Considerations

### Touch vs Click

Desktop has mouse (point click); mobile has touch (area + duration). The `Backend` trait abstracts this:

```rust
// Backend event translation:
// Mouse click at (100, 100) → Click event
// Touch tap at (100±50, 100±50) → Click event
// Touch drag start-end → Drag event

// App view function doesn't know the difference:
fn view(app: &App) -> El<App> {
    button("Tap me", |app| app.count += 1)
    // Same code works on desktop and mobile
}
```

### Screen Orientation

Mobile devices rotate. The Backend must handle orientation changes:

```rust
// iOS: UIInterfaceOrientation changes
// Android: Configuration.orientation changes
// Signal via geometry change event (width/height swap)
```

### Safe Area Insets

Mobile has notches, dynamic islands, home indicators. The view must respect safe area:

```rust
// iOS: UIWindow.safeAreaLayoutGuide
// Android: WindowInsets for status bar, navigation bar
// Passed as padding to top-level element
```

### Input Method Editor (IME)

Mobile keyboards are software. Text input requires IME handling:

```rust
// iOS: UITextInputMode, UIKeyboardAppearance
// Android: InputMethodManager
// Coordinate with text widget for soft keyboard appearance
```

---

## Cross-Module Coordination

### 1. Event Flow (Touch Input)

```
UITouch / MotionEvent (iOS/Android)
           ↓
    Backend::pump()  (ios.rs / android.rs)
           ↓
    Vec<Event> (rui Event enum)
           ↓
    Surface::draw() (shell/mod.rs)
           ↓
    view() function (user code)
           ↓
    Handler callback (updates state)
```

**Key contract:** `pump()` must translate device coordinates (physical pixels) to logical coordinates (DPI-independent), accounting for safe area insets.

### 2. Rendering Pipeline

```
view() → El<State>
       ↓
layout() (layout engine)
       ↓
paint() (paint primitives)
       ↓
Canvas::draw() (pixel buffer)
       ↓
Backend::present() (Metal CALayer / Vulkan)
       ↓
screen update (native display server)
```

**Key implementation:** `present()` must handle Metal on iOS (GPU-accelerated), Vulkan on Android (if available) or software rendering.

### 3. Appearance & Lifecycle

```
UITraitCollection.userInterfaceStyle (iOS)
UIConfiguration.uiMode (Android)
       ↓
Backend::appearance()
       ↓
Paint role selection
       ↓
Canvas pixels (light or dark)
```

**Lifecycle:** App must respond to appearance changes without full restart:

```rust
// iOS: traitCollectionDidChange()
// Android: onConfigurationChanged()
// Signal: re-render next frame without state loss
```

---

## Regression Prevention (Key Invariants)

1. **Backend trait contract:** All 6 methods must be implemented identically on all platforms (including mobile).

2. **Touch-to-click translation:** Touch events must map to the same `Click` and `Drag` events as desktop, or handlers will behave differently.

3. **DPI scaling:** Device pixels ÷ scale = logical units (same formula as desktop).

4. **Appearance consistency:** Must not change between frames unless signaled by system.

5. **Safe area respect:** Top-level element must inset by safe area (top, bottom, left, right).

6. **Lifecycle coherence:** State must persist across app suspension/resumption without losing memory.

7. **Event ordering:** Touch events must arrive in order (no reordering by platform layer).

8. **Timeout semantics:** `pump()` must return within specified timeout, unblocking frame loop.

---

## Development Roadmap

### Phase 1: iOS Backend Foundation
- [ ] Implement IosBackend struct with 6 Backend trait methods
- [ ] Add iOS feature gate to Cargo.toml
- [ ] Update platform selector in src/shell/platform/mod.rs
- [ ] Verify iOS target compiles (may require dummy FFI stubs)

### Phase 2: iOS Enhancement + Android Foundation
- [ ] Add touch event translation to iOS pump()
- [ ] Implement DPI detection (UIScreen.nativeScale)
- [ ] Implement appearance detection (UITraitCollection)
- [ ] Create android.rs with same 6 Backend trait methods
- [ ] Add Android feature gate to Cargo.toml
- [ ] Add touch event translation to Android pump()

### Phase 3: Integration & Verification
- [ ] Create ios_integration.rs (16+ tests)
- [ ] Create ios_parity.rs (10+ tests)
- [ ] Create android_integration.rs (16+ tests)
- [ ] Create android_parity.rs (10+ tests)
- [ ] Update setup.rs to verify 8+ backend selections
- [ ] Run on real iOS device (iPhone/iPad)
- [ ] Run on real Android device (phone/tablet)
- [ ] Document mobile-specific setup in CLAUDE.md

---

## Testing Strategy

### Phase 1 (Compile-time verification)
```bash
# Build iOS target (requires Apple Silicon Mac + Xcode)
cargo build --target aarch64-apple-ios --features ios

# Build Android target (requires Android NDK)
cargo build --target aarch64-linux-android --features android

# Verify core code unchanged
cargo test --lib
```

### Phase 2 (Functional tests)
```bash
# iOS integration tests (on simulator or real device)
cargo test --test ios_integration --target aarch64-apple-ios --features ios

# Android integration tests (on emulator or real device)
cargo test --test android_integration --target aarch64-linux-android --features android
```

### Phase 3 (Parity verification)
```bash
# Render same scene on iOS, Android, and desktop
# Compare PNG output pixel-by-pixel (after scaling)
# Assert: zero differing pixels within tolerance (due to platform rendering differences)

cargo test --test ios_parity --target aarch64-apple-ios --features ios
cargo test --test android_parity --target aarch64-linux-android --features android
```

---

## Challenges & Solutions

### Challenge 1: FFI Complexity

**Problem:** iOS and Android require substantial Objective-C and Java FFI.

**Solution:** Phase 1 uses stubs (no actual Objective-C/Java calls). Phase 2 introduces careful FFI bindings with thorough documentation.

### Challenge 2: Platform Variations

**Problem:** iOS and Android have different APIs, lifecycles, and graphics APIs.

**Solution:** Platform-isolated modules (ios.rs, android.rs) contain all platform-specific code. Above `Backend` trait, code is unified.

### Challenge 3: Testing Without Real Devices

**Problem:** Simulators don't always match real device behavior (touch sensitivity, performance, rendering).

**Solution:** Phase 3 includes real device testing checklist. Parity tests run on both simulator and real device.

### Challenge 4: Graphics API Selection

**Problem:** iOS prefers Metal; Android supports Vulkan, OpenGL, or software rendering.

**Solution:** Backend::present() abstracts graphics layer. Each platform chooses best option.

---

## Template: Mobile Backend Pattern

This pattern extends Recipe 2 to new mobile platforms in future:

1. **Create platform module:** `src/shell/platform/new_mobile.rs`
2. **Implement Backend trait** (6 methods)
3. **Add to platform selector** (`src/shell/platform/mod.rs`):
   ```rust
   #[cfg(target_os = "new_os")]
   #[path = "new_mobile.rs"]
   mod backend;
   ```
4. **Create integration tests:** `tests/new_mobile_integration.rs`
5. **Create parity tests:** `tests/new_mobile_parity.rs`
6. **Document setup** in CLAUDE.md

---

## Summary

The mobile backend demonstrates that the Recipe 2 pattern is truly universal. iOS, Android, desktop, web, and future platforms all:

- Implement identical Backend trait
- Follow same three-phase architecture
- Can coexist via feature gating and platform selectors
- Share coordinate contracts and event translation
- Are platform-isolated within single files

STEP 23 establishes parity between mobile and desktop, ensuring the same UI code runs everywhere—a powerful feature for cross-platform Rust applications.

---

## Next Steps After STEP 23

Once mobile backends are complete:

- **STEP 24:** Gesture recognition (swipe, pinch, long press)
- **STEP 25:** Native dialogs (alert, file picker, camera)
- **STEP 26:** Platform-specific optimizations (async rendering, background tasks)
- **STEP 27:** App store deployment (TestFlight, Google Play)

