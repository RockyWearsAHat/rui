# Recipe 2: X11 Backend — Event Translation

Reference document for Recipe 2 (X11 Backend Implementation). This file documents how X11 event types are translated to rui Event types.

## Translation Overview

X11 delivers raw protocol events; rui requires platform-agnostic Events. The translation happens in x11.rs::pump() before returning Vec<Event>.

**Translation layers:**

```
X11 Protocol Event
    ↓
x11.rs::pump()  ← (translate here)
    ↓
rui Event enum
    ↓
input.rs::Input  ← (resolve key meanings)
    ↓
paint.rs handlers  ← (ready to use)
```

## Event Types and Translations

### 1. Pointer Motion: MotionNotify → Event::Pointer

**X11 Event:**
```c
XMotionEvent {
    int x, y;           // relative to window
    int x_root, y_root; // relative to screen
}
```

**rui Event:**
```rust
Event::Pointer {
    x: f32,         // logical units
    y: f32,         // logical units
    moved: true,    // indicates this is a movement event
    ..Default::default()
}
```

**Implementation:**
```rust
XEvent::MotionNotify(motion) => {
    events.push(Event::Pointer {
        x: motion.x as f32 / self.scale_factor,
        y: motion.y as f32 / self.scale_factor,
        moved: true,
        ..Default::default()
    });
}
```

**Notes:**
- Only fire when pointer actually moved (filter duplicate events)
- Convert device pixels to logical units by dividing by scale_factor
- moved=true distinguishes from other pointer events (pressed, released)

### 2. Pointer Press: ButtonPress → Event::Pointer

**X11 Event:**
```c
XButtonEvent {
    int button;     // 1=left, 2=middle, 3=right, 4=scroll_up, 5=scroll_down
    int x, y;
    unsigned int state;  // modifier mask (ShiftMask, ControlMask, Mod1Mask)
}
```

**rui Event:**
```rust
Event::Pointer {
    x: f32,                       // logical units
    y: f32,                       // logical units
    button: PointerButton::Primary,  // or Secondary, Middle
    pressed: true,
    modifiers: Modifiers { shift: true, control: false, alt: false },
    ..Default::default()
}
```

**Implementation:**
```rust
XEvent::ButtonPress(button) => {
    let button_kind = match button.button {
        1 => PointerButton::Primary,    // left click
        2 => PointerButton::Middle,
        3 => PointerButton::Secondary,  // right click
        _ => return,  // scroll buttons handled separately
    };
    
    events.push(Event::Pointer {
        x: button.x as f32 / self.scale_factor,
        y: button.y as f32 / self.scale_factor,
        button: button_kind,
        pressed: true,
        modifiers: decode_modifiers(button.state),
        ..Default::default()
    });
}
```

**Modifier decoding:**
```rust
fn decode_modifiers(state: unsigned int) -> Modifiers {
    Modifiers {
        shift: (state & ShiftMask) != 0,
        control: (state & ControlMask) != 0,
        alt: (state & Mod1Mask) != 0,
        // meta: (state & Mod4Mask) != 0,  // if rui supports it
    }
}
```

**Notes:**
- Button numbers must be translated: X11 1→3, 2→2, 3→1
- Modifier mask is a bitmask; decode into Modifiers struct
- No synthetic multi-click detection; X11 handles that with event count field

### 3. Pointer Release: ButtonRelease → Event::Pointer

**X11 Event:**
```c
XButtonEvent {
    int button;
    int x, y;
    unsigned int state;  // modifiers BEFORE release (includes button)
}
```

**rui Event:**
```rust
Event::Pointer {
    x: f32,
    y: f32,
    button: PointerButton::Primary,
    released: true,
    modifiers: Modifiers { /* before release */ },
    ..Default::default()
}
```

**Implementation:**
```rust
XEvent::ButtonRelease(button) => {
    let button_kind = match button.button {
        1 => PointerButton::Primary,
        2 => PointerButton::Middle,
        3 => PointerButton::Secondary,
        _ => return,
    };
    
    // Note: state includes the button being released
    let mut modifiers = decode_modifiers(button.state);
    // But rui expects modifiers AFTER release, so clear the button bit
    // Actually, rui doesn't track button state in modifiers, so this is safe as-is
    
    events.push(Event::Pointer {
        x: button.x as f32 / self.scale_factor,
        y: button.y as f32 / self.scale_factor,
        button: button_kind,
        released: true,
        modifiers,
        ..Default::default()
    });
}
```

**Notes:**
- released=true distinguishes from pressed events
- X11 state includes the button being released; rui expects pre-release state
- Coordinate conversion same as press

### 4. Pointer Scroll: Button 4/5 → Event::Scroll

**X11 Event:**
```c
XButtonEvent {
    int button;     // 4=scroll_up, 5=scroll_down
    int x, y;
}
```

**rui Event:**
```rust
Event::Scroll {
    x: f32,
    y: f32,
    delta_x: 0.0,
    delta_y: -1.0,  // negative = down, positive = up
}
```

**Implementation:**
```rust
XEvent::ButtonPress(button) if button.button == 4 || button.button == 5 => {
    let delta_y = if button.button == 4 { 1.0 } else { -1.0 };
    events.push(Event::Scroll {
        x: button.x as f32 / self.scale_factor,
        y: button.y as f32 / self.scale_factor,
        delta_x: 0.0,
        delta_y,
    });
}
```

**Notes:**
- X11 button 4 = scroll up = positive delta_y
- X11 button 5 = scroll down = negative delta_y
- Modern X11 may use XGenericEvent for smooth scrolling (todo for enhancement phase)

### 5. Keyboard Press: KeyPress → Event::Key

**X11 Event:**
```c
XKeyEvent {
    KeyCode keycode;        // position (1-255), not meaning
    unsigned int state;     // modifiers
    // Use XkbLookupKeySym to get keysym from keycode
}
```

**rui Event:**
```rust
Event::Key {
    key: Key::A,            // resolved meaning, e.g., Key::A, Key::ArrowUp
    keycode: KeyCode::new(38),  // position, for forwarding to IME
    pressed: true,
    modifiers: Modifiers { shift: true, control: false, alt: false },
}
```

**Implementation:**
```rust
XEvent::KeyPress(key) => {
    let keysym = unsafe {
        xkb_keycode_to_keysym(key.keycode, self.xkb_state)
    };
    
    let rui_key = translate_keysym_to_key(keysym);
    
    events.push(Event::Key {
        key: rui_key,
        keycode: KeyCode::new(key.keycode as u32),
        pressed: true,
        modifiers: decode_modifiers(key.state),
    });
}
```

**KeySym → Key translation table:**

| X11 KeySym | rui Key | Notes |
|------------|---------|-------|
| XK_a ... XK_z | Key::A ... Key::Z | Ignores Shift; modifiers field carries it |
| XK_0 ... XK_9 | Key::Digit0 ... Key::Digit9 | Same |
| XK_Return | Key::Enter | Named key |
| XK_Tab | Key::Tab | |
| XK_Escape | Key::Escape | |
| XK_BackSpace | Key::Backspace | |
| XK_Delete | Key::Delete | |
| XK_space | Key::Space | |
| XK_Up / Down / Left / Right | Key::ArrowUp / Down / Left / Right | |
| XK_Home | Key::Home | |
| XK_End | Key::End | |
| XK_Page_Up / Down | Key::PageUp / Down | |
| XK_F1 ... XK_F12 | Key::F1 ... Key::F12 | Function keys |
| XK_Shift_L / Shift_R | Key::Shift | |
| XK_Control_L / Control_R | Key::Control | |
| XK_Alt_L / Alt_R | Key::Alt | |
| (unknown) | Key::Unknown | Fallback |

**Notes:**
- X11 delivers KeyCode (position); must resolve to keysym (meaning) using XKB
- rui Key should match the key's meaning, not the physical position
- Modifiers should track shift/control/alt state
- keycode field used by IME composition, not UI handlers

### 6. Keyboard Release: KeyRelease → Event::Key

**X11 Event:**
```c
XKeyEvent {
    KeyCode keycode;
    unsigned int state;  // modifiers INCLUDING the key being released
}
```

**rui Event:**
```rust
Event::Key {
    key: Key::A,
    keycode: KeyCode::new(38),
    released: true,
    modifiers: Modifiers { shift: true, ... },
}
```

**Implementation:**
```rust
XEvent::KeyRelease(key) => {
    let keysym = unsafe {
        xkb_keycode_to_keysym(key.keycode, self.xkb_state)
    };
    
    let rui_key = translate_keysym_to_key(keysym);
    
    // Note: state includes the key being released; rui expects pre-release state
    // For key release, just report the key without the released bit in modifiers
    
    events.push(Event::Key {
        key: rui_key,
        keycode: KeyCode::new(key.keycode as u32),
        released: true,
        modifiers: decode_modifiers(key.state),
    });
}
```

**Notes:**
- KeyRelease always reports the key being released
- rui receives released=true to distinguish from press events
- Every KeyPress must have a corresponding KeyRelease (enforced by input.rs)

### 7. Window Resize: ConfigureNotify → Event::Resized

**X11 Event:**
```c
XConfigureEvent {
    int width, height;   // device pixels
    int x, y;            // window position (usually ignored)
}
```

**rui Event:**
```rust
Event::Resized {
    width: 800u32,   // logical units
    height: 600u32,
}
```

**Implementation:**
```rust
XEvent::ConfigureNotify(config) => {
    let width = config.width as f32 / self.scale_factor;
    let height = config.height as f32 / self.scale_factor;
    
    // Avoid sending duplicate resize events
    if (width as u32, height as u32) != (self.cached_width, self.cached_height) {
        self.cached_width = width as u32;
        self.cached_height = height as u32;
        
        events.push(Event::Resized {
            width: width as u32,
            height: height as u32,
        });
    }
}
```

**Notes:**
- Convert device pixels to logical units
- Coalesce multiple ConfigureNotify events (filter duplicates)
- Triggers layout recalculation in app.rs

### 8. Focus Change: FocusIn/FocusOut → Event::FocusChanged

**X11 Event:**
```c
XFocusChangeEvent {
    int mode;  // Normal, Grab, Ungrab
}
```

**rui Event:**
```rust
Event::FocusChanged {
    focused: true,  // true for FocusIn, false for FocusOut
}
```

**Implementation:**
```rust
XEvent::FocusIn(_) => {
    events.push(Event::FocusChanged { focused: true });
}
XEvent::FocusOut(_) => {
    events.push(Event::FocusChanged { focused: false });
}
```

**Notes:**
- Window-level focus; element-level focus handled by memory.rs
- Triggers IME activation/deactivation

## Testing Event Translation

**Unit test template:**
```rust
#[test]
fn button_press_translates_to_pointer_event() {
    let button_press = XButtonEvent {
        button: 1,  // left
        x: 100,
        y: 200,
        state: 0,  // no modifiers
    };
    
    let event = translate_x11_event(button_press, scale_factor=1.0);
    
    assert_eq!(event.button, PointerButton::Primary);
    assert_eq!(event.x, 100.0);
    assert_eq!(event.y, 200.0);
    assert!(event.pressed);
}
```

**Integration test template:**
```bash
cargo test --test x11_integration -- event_translation
```

## Verification Checklist

- [ ] MotionNotify → moved events only when pointer changes
- [ ] ButtonPress/Release → correct button number mapping (1→Primary, 2→Middle, 3→Secondary)
- [ ] Button 4/5 → scroll events, not pointer press/release
- [ ] KeyPress/Release → keysym lookup via XKB, correct Key translation
- [ ] Modifiers → correctly decoded from state mask
- [ ] Coordinates → always in logical units (device / scale_factor)
- [ ] Duplicates → ConfigureNotify coalesced, no repeated events
- [ ] Focus → FocusIn/Out generate FocusChanged events
- [ ] All translations produce valid rui Events (never None, never panic)

## Reference

Recipe 2 commits implementing event translation:

- Phase 2 (1220 lines): c42c0f05b3d75976665377a16257c36c472debc1 — Full event translation
- Phase 3 (1321 lines): 80e3003563c26952e4d63c52d8eb8f5052cb463c — Event integration + parity

See these commits for real-world event handling:
```bash
git show c42c0f05:src/shell/platform/x11.rs | grep -A10 "fn pump"
```
