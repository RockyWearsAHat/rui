# X11 Event Translation: Mapping X11 Events to rui Input Types

## Overview

The X11 backend receives X11 events and translates them into rui's unified `Event` type.

## Event Type Mapping

### X11 → rui Translation Table

| X11 Event | rui Event Type | Coordinate Translation |
|-----------|----------------|----------------------|
| ButtonPress (Button1) | `Click` | device → logical |
| ButtonRelease | Release | device → logical |
| MotionNotify | `Drag`/`Hover` | device → logical + delta |
| KeyPress | `Key` | X11 KeySym → rui Key enum |
| KeyRelease | Release | X11 KeySym → rui Key enum |
| Expose | Redraw | dirty region |
| FocusIn | `Focus` | — |
| FocusOut | Blur | — |
| EnterWindow | `Hover` | device → logical |
| LeaveWindow | Blur | — |
| ConfigureNotify | Resize | new dimensions |

## Detailed Translations

### Mouse Button Events

#### X11 ButtonPress → rui Click

```rust
fn translate_button_press(xevent: &XButtonEvent, scale: f32) -> Event {
    let logical_x = (xevent.x as f32) / scale;
    let logical_y = (xevent.y as f32) / scale;
    
    let button = match xevent.button {
        1 => Button::Left,
        2 => Button::Middle,
        3 => Button::Right,
        _ => Button::Other(xevent.button),
    };
    
    Event::Click {
        position: Point::new(logical_x, logical_y),
        button,
        modifiers: translate_modifiers(xevent.state),
    }
}
```

### Motion Events

#### X11 MotionNotify → rui Drag (or Hover)

```rust
fn translate_motion_notify(
    xevent: &XMotionEvent,
    scale: f32,
    previous_position: Option<Point>,
) -> Event {
    let current_x = (xevent.x as f32) / scale;
    let current_y = (xevent.y as f32) / scale;
    
    // Check if a button is pressed
    let is_dragging = (xevent.state & (Button1Mask | Button2Mask | Button3Mask)) != 0;
    
    if is_dragging {
        let delta = match previous_position {
            Some(prev) => (current_x - prev.x, current_y - prev.y),
            None => (0.0, 0.0),
        };
        
        Event::Drag {
            from: previous_position.unwrap_or(Point::new(current_x, current_y)),
            to: Point::new(current_x, current_y),
            delta: (delta.0, delta.1),
            button: get_pressed_button(xevent.state),
        }
    } else {
        Event::Hover {
            position: Point::new(current_x, current_y),
        }
    }
}
```

### Keyboard Events

#### X11 KeyPress → rui Key

```rust
fn translate_keysym(keysym: KeySym) -> Option<Key> {
    match keysym {
        XK_Return => Some(Key::Enter),
        XK_Escape => Some(Key::Escape),
        XK_BackSpace => Some(Key::Backspace),
        XK_Tab => Some(Key::Tab),
        XK_Left => Some(Key::ArrowLeft),
        XK_Right => Some(Key::ArrowRight),
        XK_Up => Some(Key::ArrowUp),
        XK_Down => Some(Key::ArrowDown),
        XK_Home => Some(Key::Home),
        XK_End => Some(Key::End),
        XK_Page_Up => Some(Key::PageUp),
        XK_Page_Down => Some(Key::PageDown),
        XK_Delete => Some(Key::Delete),
        XK_shift_L | XK_shift_R => Some(Key::Shift),
        XK_control_L | XK_control_R => Some(Key::Control),
        XK_alt_L | XK_alt_R => Some(Key::Alt),
        _ if keysym >= XK_space && keysym <= XK_asciitilde => {
            Some(Key::Character(keysym as u8 as char))
        }
        _ => None,
    }
}

fn translate_key_press(xevent: &XKeyEvent, scale: f32) -> Event {
    let keysym = XLookupKeysym(xevent, 0);
    
    match translate_keysym(keysym) {
        Some(key) => Event::Key {
            key,
            modifiers: translate_modifiers(xevent.state),
        },
        None => Event::Unknown,
    }
}
```

#### Modifier Key Translation

```rust
fn translate_modifiers(state: u32) -> Modifiers {
    Modifiers {
        shift: (state & ShiftMask) != 0,
        control: (state & ControlMask) != 0,
        alt: (state & Mod1Mask) != 0,      // Mod1 is typically Alt
        meta: (state & Mod4Mask) != 0,     // Mod4 is typically Super/Meta
    }
}
```

## Phase-by-Phase Event Translation

### Phase 1: Foundation (Commit a67d578)
**Gate:** ButtonPress/ButtonRelease, MotionNotify (basic hover), KeyPress for printable characters.

### Phase 2: Enhancement (Commit c42c0f0)
**Gate:** All keyboard keys (special + printable), modifiers (Shift, Control, Alt), appearance events.

### Phase 3: Integration (Commits 80e3003–84ade0e)
**Gate:** All events translated correctly in turn() loop; event stream is consistent and complete.

## Testing Event Translation

```bash
# Run interaction tests
cargo test --test interaction -- --nocapture

# Manual test: Run counter and verify button clicks work
DISPLAY=:0 cargo run -p rui --example counter

# Verify keyboard input works
# - Click Increment; counter should increase
# - Press arrow keys; focus should move
# - Drag slider; value should change smoothly
```

## Event Translation Regression Prevention

```bash
# After changes to event translation code:
cargo test --test x11_backend_phases -- event_translation
cargo test --test interaction -- --nocapture
```

## Summary Table: Event Type Coverage

| Event Category | X11 Event | rui Event | Phase |
|---|---|---|---|
| Mouse | ButtonPress | Click | 1 |
| Mouse | ButtonRelease | Release | 1 |
| Mouse | MotionNotify | Drag/Hover | 1, 3 |
| Keyboard | KeyPress (special) | Key | 2 |
| Keyboard | KeyPress (char) | Key | 1, 2 |
| Modifiers | state flags | Modifiers | 2 |
| Window | FocusIn | Focus | 2 |
| Window | FocusOut | Blur | 2 |
| Window | Expose | Redraw | 1 |
