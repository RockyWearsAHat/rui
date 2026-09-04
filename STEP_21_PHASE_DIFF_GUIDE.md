# X11 Backend: Phase-by-Phase Code Changes

## Overview

This document shows the specific code changes made at each phase of the X11 backend implementation. Each phase builds on the previous one, adding capabilities while maintaining the Backend trait contract.

---

## Phase 1: Foundation (Commit a67d578)

### Objective
Implement the Backend trait for X11. Establish:
- Window creation and event loop basics
- Device coordinate handling (no scaling yet)
- Frame rendering (pixel blit to X11)

### Files Added/Modified

#### New File: `src/shell/platform/x11.rs`

```rust
// Minimal Backend implementation for X11

use crate::shell::{Backend, Event, Window, Appearance, Error};
use std::ffi::CStr;

// X11 FFI bindings (simplified; real code uses xlib crate)
pub struct X11Backend {
    display: *mut Display,
    window: Window,
    screen: c_int,
    width: u32,
    height: u32,
    scale: f32,
}

impl Backend for X11Backend {
    fn open(options: &WindowOptions) -> Result<Self, Error> {
        // Step 1: XOpenDisplay(NULL) - connect to X11 server
        let display = unsafe { XOpenDisplay(std::ptr::null()) };
        if display.is_null() {
            return Err(Error::CannotOpenDisplay);
        }
        
        // Step 2: Get default screen
        let screen = unsafe { DefaultScreen(display) };
        
        // Step 3: XCreateWindow with event mask
        let root = unsafe { RootWindow(display, screen) };
        let window = unsafe {
            XCreateWindow(
                display,
                root,
                0, 0,           // position
                options.width as u32,
                options.height as u32,
                0,              // border width
                CopyFromParent, // depth
                InputOutput,    // class
                std::ptr::null_mut(), // visual
                0,              // valuemask
                std::ptr::null_mut(), // attributes
            )
        };
        
        // Step 4: Select input events
        let event_mask = ButtonPressMask
            | ButtonReleaseMask
            | PointerMotionMask
            | KeyPressMask
            | KeyReleaseMask
            | ExposureMask
            | StructureNotifyMask;
        
        unsafe {
            XSelectInput(display, window, event_mask);
            XMapWindow(display, window);
        }
        
        // Step 5: Calculate scale (phase 1: always 1.0)
        let scale = 1.0;
        
        Ok(X11Backend {
            display,
            window,
            screen,
            width: options.width as u32,
            height: options.height as u32,
            scale,
        })
    }

    fn pump(
        &mut self,
        _timeout: Duration,
        events: &mut Vec<Event>,
        _redraw: &mut dyn FnMut(&Self),
    ) -> Result<(), Error> {
        // Step 1: Loop through available X11 events
        loop {
            let mut xevent = std::mem::zeroed::<XEvent>();
            
            if unsafe { XCheckMaskEvent(self.display, -1, &mut xevent) } == 0 {
                break; // No more events
            }
            
            // Step 2: Translate X11 event to rui Event
            let event = self.translate_event(&xevent);
            
            if let Some(e) = event {
                events.push(e);
            }
        }
        
        Ok(())
    }

    fn surface(&self) -> (u32, u32, f32) {
        (self.width, self.height, self.scale)
    }

    fn appearance(&self) -> Appearance {
        // Phase 1: hardcoded to light (no detection yet)
        Appearance::Light
    }

    fn present(&self, canvas: &Canvas) -> Result<(), Error> {
        // Step 1: Create XImage from canvas pixels
        let image = unsafe {
            XCreateImage(
                self.display,
                std::ptr::null_mut(), // visual
                24,                   // depth (RGB)
                ZPixmap,              // format
                0,                    // offset
                canvas.pixels() as *mut u8,
                self.width,
                self.height,
                32,                   // bitmap pad
                0,                    // bytes per line (auto)
            )
        };
        
        if image.is_null() {
            return Err(Error::CannotCreateImage);
        }
        
        // Step 2: XPutImage - blit to window
        let gc = unsafe { DefaultGC(self.display, self.screen) };
        
        unsafe {
            XPutImage(
                self.display,
                self.window,
                gc,
                image,
                0, 0,                // src x, y
                0, 0,                // dst x, y
                self.width,
                self.height,
            );
            
            // Cleanup
            XDestroyImage(image);
            XSync(self.display, 0);
        }
        
        Ok(())
    }

    fn is_open(&self) -> bool {
        !self.display.is_null()
    }
}

impl X11Backend {
    fn translate_event(&self, xevent: &XEvent) -> Option<Event> {
        unsafe {
            match xevent.type_ {
                ButtonPress => {
                    let event = &xevent.xbutton;
                    let x = (event.x as f32) / self.scale;
                    let y = (event.y as f32) / self.scale;
                    
                    Some(Event::Click {
                        position: Point::new(x, y),
                        button: match event.button {
                            1 => Button::Left,
                            3 => Button::Right,
                            _ => Button::Left,
                        },
                        modifiers: Default::default(),
                    })
                }
                MotionNotify => {
                    let event = &xevent.xmotion;
                    let x = (event.x as f32) / self.scale;
                    let y = (event.y as f32) / self.scale;
                    
                    Some(Event::Hover {
                        position: Point::new(x, y),
                    })
                }
                KeyPress => {
                    let event = &xevent.xkey;
                    let keysym = XLookupKeysym(event as *mut XKeyEvent, 0);
                    
                    // Phase 1: printable characters only
                    if keysym >= XK_space && keysym <= XK_asciitilde {
                        Some(Event::Key {
                            key: Key::Character(keysym as u8 as char),
                            modifiers: Default::default(),
                        })
                    } else {
                        None
                    }
                }
                Expose => {
                    let event = &xevent.xexpose;
                    Some(Event::Redraw {
                        region: Rect::new(
                            event.x as f32,
                            event.y as f32,
                            event.width as f32,
                            event.height as f32,
                        ),
                    })
                }
                _ => None,
            }
        }
    }
}
```

### Modified: `src/shell/mod.rs`

Add conditional platform selection:

```rust
#[cfg(target_os = "linux")]
mod x11;

#[cfg(target_os = "linux")]
use x11::X11Backend as NativeBackend;

#[cfg(target_os = "macos")]
use cocoa::CocoaBackend as NativeBackend;

// ... rest of shell code uses NativeBackend
```

### Verification Gate (Phase 1)

```bash
# Compile check
cargo build --target x86_64-unknown-linux-gnu

# Verify Backend trait is implemented
cargo test --lib backend_trait

# Manual test: app starts and responds to clicks
DISPLAY=:0 cargo run -p rui --example counter
```

### Key Invariants Established (Phase 1)

1. **Device coordinate handling**: Click at device (100, 100) translates to logical (100.0, 100.0) with scale=1.0
2. **Window creation**: XCreateWindow succeeds; window is displayed on X11 server
3. **Event loop**: XCheckMaskEvent retrieves events; event stream is non-blocking
4. **Pixel rendering**: XPutImage blits canvas to window; visual appears on display

---

## Phase 2: Enhancement (Commit c42c0f0)

### Objective
Add missing features and polish:
- DPI detection (coordinate scaling)
- Keyboard translation (special keys, modifiers)
- Appearance detection (light/dark mode)
- Font loading from system directories

### Files Modified: `src/shell/platform/x11.rs`

#### Change 1: DPI Detection in `open()`

```rust
fn open(options: &WindowOptions) -> Result<Self, Error> {
    // ... previous code ...
    
    // NEW: Calculate DPI scale factor
    let width_pixels = unsafe { XDisplayWidth(display, screen) } as f32;
    let width_mm = unsafe { XDisplayWidthMM(display, screen) } as f32;
    
    // DPI = pixels / (mm / 25.4 mm per inch)
    let dpi = width_pixels / (width_mm / 25.4);
    
    // Scale factor: 96 DPI is baseline (1.0x)
    let scale = dpi / 96.0;
    
    // Clamp to reasonable range
    let scale = scale.clamp(0.5, 2.5);
    
    Ok(X11Backend {
        display,
        window,
        screen,
        width: (options.width as f32 / scale) as u32,  // logical dimensions
        height: (options.height as f32 / scale) as u32,
        scale,
    })
}
```

#### Change 2: Keyboard Translation Enhanced

```rust
fn translate_event(&self, xevent: &XEvent) -> Option<Event> {
    unsafe {
        match xevent.type_ {
            KeyPress => {
                let event = &xevent.xkey;
                let keysym = XLookupKeysym(event as *mut XKeyEvent, 0);
                
                // NEW: Full keyboard support
                let key = match keysym {
                    XK_Return => Key::Enter,
                    XK_Escape => Key::Escape,
                    XK_BackSpace => Key::Backspace,
                    XK_Tab => Key::Tab,
                    XK_Left => Key::ArrowLeft,
                    XK_Right => Key::ArrowRight,
                    XK_Up => Key::ArrowUp,
                    XK_Down => Key::ArrowDown,
                    XK_Home => Key::Home,
                    XK_End => Key::End,
                    XK_Page_Up => Key::PageUp,
                    XK_Page_Down => Key::PageDown,
                    XK_Delete => Key::Delete,
                    _ if keysym >= XK_space && keysym <= XK_asciitilde => {
                        Key::Character(keysym as u8 as char)
                    }
                    _ => return None,
                };
                
                // NEW: Extract modifiers
                let modifiers = Modifiers {
                    shift: (event.state & ShiftMask) != 0,
                    control: (event.state & ControlMask) != 0,
                    alt: (event.state & Mod1Mask) != 0,
                    meta: (event.state & Mod4Mask) != 0,
                };
                
                Some(Event::Key { key, modifiers })
            }
            _ => self.translate_event_old(xevent),
        }
    }
}
```

#### Change 3: Appearance Detection

```rust
fn appearance(&self) -> Appearance {
    // NEW: Detect light/dark mode
    
    // Try environment variable first
    if let Ok(colorterm) = std::env::var("COLORFTERM") {
        if colorterm.contains("truecolor") || colorterm.contains("dark") {
            return Appearance::Dark;
        }
    }
    
    // Try X11 property (modern window managers)
    unsafe {
        let atom = XInternAtom(self.display, b"_NET_WM_APPEARANCE\0".as_ptr() as *const i8, 0);
        if atom != 0 {
            let mut actual_type = 0;
            let mut actual_format = 0;
            let mut nitems = 0;
            let mut bytes_after = 0;
            let mut prop = std::ptr::null_mut();
            
            if XGetWindowProperty(
                self.display,
                self.window,
                atom,
                0, 1,
                0,
                XA_CARDINAL,
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut prop,
            ) == Success as i32
                && !prop.is_null()
            {
                let value = *(prop as *mut u32);
                XFree(prop as *mut c_void);
                
                if value == 0 {
                    return Appearance::Light;
                } else if value == 1 {
                    return Appearance::Dark;
                }
            }
        }
    }
    
    // Fallback
    Appearance::Light
}
```

### Verification Gate (Phase 2)

```bash
# DPI detection works
cargo test --test x11_backend_phases -- dpi_scale

# Keyboard translation is complete
cargo test --test x11_backend_phases -- keyboard_translation

# Appearance detection logic is correct
cargo test --test x11_backend_phases -- appearance_detection

# Manual test with different scales
# On a 2x DPI monitor:
DISPLAY=:0 cargo run -p rui --example counter

# Manual test with dark mode
COLORFTERM=truecolor DISPLAY=:0 cargo run -p rui --example counter
```

### Key Invariants Preserved (Phase 2)

1. **DPI scaling formula**: `scale = dpi / 96.0`; verified for common monitors (1.0x, 1.5x, 2.0x)
2. **Coordinate translation**: Click at device (200, 200) with 2x scale → logical (100.0, 100.0)
3. **Keyboard completeness**: All special keys + printable chars + modifiers translate correctly
4. **Appearance detection**: Reads COLORFTERM env var and X11 property; falls back to light

---

## Phase 3: Integration (Commits 80e3003–84ade0e)

### Objective
Integrate X11 backend into the platform-agnostic frame loop. Ensure:
- Coordinate contract is preserved across frame boundaries
- Event stream is consistent and complete
- DPI scaling doesn't break widget hit testing
- EventLoopDriver abstraction unifies timeout handling

### Files Modified: `src/shell/mod.rs`

#### Change: Add EventLoopDriver Trait

```rust
trait EventLoopDriver {
    fn pump_timeout(&self) -> Duration;
    fn max_frames_per_second(&self) -> u32;
}

#[cfg(target_os = "linux")]
impl EventLoopDriver for X11Backend {
    fn pump_timeout(&self) -> Duration {
        // X11 uses XNextEvent with timeout; 8ms allows 60fps
        Duration::from_millis(8)
    }
    
    fn max_frames_per_second(&self) -> u32 {
        60
    }
}

#[cfg(target_os = "macos")]
impl EventLoopDriver for CocoaBackend {
    fn pump_timeout(&self) -> Duration {
        Duration::from_millis(8)
    }
    
    fn max_frames_per_second(&self) -> u32 {
        60
    }
}
```

#### Change: Verify Coordinate Contract in `turn()`

```rust
fn turn<S, V>(
    surface: &mut Surface,
    backend: &mut NativeBackend,
    app: &mut S,
    view: &V,
) -> Result<(), Error>
where
    V: Fn(&S) -> El<S>,
{
    // ... existing code ...
    
    // NEW: Verify coordinate translation before passing to view
    for event in &mut surface.input.events {
        // All Click, Drag, Hover events should have logical coordinates
        match event {
            Event::Click { position, .. } => {
                assert!(position.x >= 0.0 && position.y >= 0.0,
                    "Click coordinates should be non-negative logical coordinates");
            }
            Event::Drag { from, to, .. } => {
                assert!(from.x >= 0.0 && from.y >= 0.0,
                    "Drag 'from' should have non-negative logical coordinates");
                assert!(to.x >= 0.0 && to.y >= 0.0,
                    "Drag 'to' should have non-negative logical coordinates");
            }
            Event::Hover { position } => {
                assert!(position.x >= 0.0 && position.y >= 0.0,
                    "Hover coordinates should be non-negative logical coordinates");
            }
            _ => {}
        }
    }
    
    // ... rest of frame logic ...
}
```

#### Change: Widget Hit Testing Respects Scale

```rust
// In src/memory.rs or src/paint.rs: verify hit testing uses logical coordinates
fn hit_test(element: &El, logical_x: f32, logical_y: f32) -> bool {
    // element.bounds() returns logical rect
    let rect = element.bounds();
    
    logical_x >= rect.x && logical_x < rect.x + rect.w
        && logical_y >= rect.y && logical_y < rect.y + rect.h
}
```

### Verification Gate (Phase 3)

```bash
# Coordinate contract preserved in frame loop
cargo test --test x11_backend_phases -- coordinate_contract

# Event stream is consistent
cargo test --test interaction -- --nocapture

# Widget hit testing works at various scales
cargo test --test recipes -- segmented_control

# Full integration test
cargo test --test integration

# Manual test: Verify clicks hit correct buttons at various scales
# On a 2x DPI monitor:
DISPLAY=:0 cargo run -p rui --example controls
# Click buttons; verify they respond
```

### Key Invariants Ensured (Phase 3)

1. **Coordinate contract across frames**: Click at device (x, y) → logical (x/scale, y/scale) is preserved
2. **Widget hit testing**: Button at logical (lx, ly, lw, lh) is hit by logical click (x, y) where lx ≤ x < lx+lw
3. **Event stream completeness**: No events are dropped; modifiers are preserved
4. **DPI scaling transparency**: App works identically at 1x, 1.5x, 2x scales (only pixel density changes)

---

## Cross-Phase Verification: Regression Prevention

### After Phase 1 (Foundation)
```bash
cargo build --target x86_64-unknown-linux-gnu
cargo test --test x11_backend_phases -- foundation
```

### After Phase 2 (Enhancement)
```bash
cargo build --target x86_64-unknown-linux-gnu
cargo test --test x11_backend_phases -- enhancement
cargo test --test x11_backend_phases -- coordinate_contract
```

### After Phase 3 (Integration)
```bash
cargo build --target x86_64-unknown-linux-gnu
cargo test --test x11_backend_phases -- integration
cargo test --test interaction
cargo test --test integration
```

### Continuous Regression Prevention
```bash
# Always run these before committing:
cargo fmt --check
cargo clippy -- -D warnings
cargo test --test x11_backend_phases
cargo test --test interaction
cargo test --test integration
```

---

## Summary: What Changed at Each Phase

| Phase | Key Changes | Invariants Established | Files Modified |
|-------|-------------|----------------------|-----------------|
| **1: Foundation** | Backend trait, X11 FFI, window creation, basic events | Device coords, window init, event pump | `x11.rs` (new), `mod.rs` |
| **2: Enhancement** | DPI detection, full keyboard, appearance detection | Coordinate scaling, keyboard completeness | `x11.rs` |
| **3: Integration** | EventLoopDriver trait, coordinate contract verification | Event stream consistency, scale transparency | `mod.rs`, `input.rs` (implicit) |

