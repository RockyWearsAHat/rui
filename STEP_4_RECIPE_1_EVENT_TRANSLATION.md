# Recipe 1: WASM Backend — Event Translation

**Document**: STEP_4_RECIPE_1_EVENT_TRANSLATION.md  
**Purpose**: Define the mapping from browser DOM events to rui Event types  
**Scope**: All input sources (mouse, touch, keyboard, wheel, composition)  
**Audience**: WASM backend implementer, event loop integration  

## Overview

The WASM backend runs in a browser which fires DOM events. These must be translated into rui's `Event` type before being consumed by the frame loop.

**Key principle**: One rui Event per user action. Multiple DOM events may combine into one rui Event (e.g., `mousedown` + `mousemove` during drag).

## Event Type Mappings

### 1. Pointer Events (Mouse and Touch)

#### Source: `mousedown`, `mouseup`, `mousemove`

**DOM MouseEvent properties**:
- `clientX`, `clientY` — viewport coordinates
- `buttons` — bitmask of pressed buttons (1=left, 2=right, 4=middle)
- `button` — which button triggered this event (0=left, 1=middle, 2=right, 3=browser-back, 4=browser-forward)
- `ctrlKey`, `shiftKey`, `altKey`, `metaKey` — modifier key states

**Transformation**:
1. Normalize `clientX/Y` to canvas coordinates (see COORDINATE_CONTRACT.md)
2. Convert canvas coordinates to logical coordinates (divide by scale_factor)
3. Extract modifier bits (shift=1, control=2, alt=4, meta=8)
4. Map DOM button number to rui button number (see table below)

**DOM Button → rui Button Mapping**:
| DOM button | Button Name | rui Mapping |
|-----------|-------------|-----------|
| 0         | Left        | 0 (primary) |
| 1         | Middle      | 1 (middle) |
| 2         | Right       | 2 (secondary) |
| 3         | Browser Back | ignored |
| 4         | Browser Forward | ignored |

**Rui Event Type**:
```rust
pub enum Event {
    Pointer {
        at: Point,
        button: u8,
        pressed: bool,
        released: bool,
        moved: bool,
        modifiers: u8,  // shift=1, control=2, alt=4, meta=8
    },
    // ... other variants
}
```

#### Implementation

**Phase 1: Basic Pointer Events**

```javascript
canvas.addEventListener('mousedown', (event) => {
    const at = transform_client_to_logical(event.clientX, event.clientY);
    const button = event.button;  // 0=left, 1=middle, 2=right
    const modifiers = get_modifiers(event);
    
    events.push({
        type: 'pointer',
        at,
        button,
        pressed: true,
        released: false,
        moved: false,
        modifiers
    });
});

canvas.addEventListener('mouseup', (event) => {
    const at = transform_client_to_logical(event.clientX, event.clientY);
    const button = event.button;
    const modifiers = get_modifiers(event);
    
    events.push({
        type: 'pointer',
        at,
        button,
        pressed: false,
        released: true,
        moved: false,
        modifiers
    });
});

canvas.addEventListener('mousemove', (event) => {
    const at = transform_client_to_logical(event.clientX, event.clientY);
    const modifiers = get_modifiers(event);
    
    // Only report movement if actually moved (avoid duplicate events)
    if (at !== last_pointer_pos) {
        events.push({
            type: 'pointer',
            at,
            button: 0,  // No specific button on move (buttons bitmask from event.buttons)
            pressed: false,
            released: false,
            moved: true,
            modifiers
        });
        last_pointer_pos = at;
    }
});

// Helper
function get_modifiers(event) {
    let modifiers = 0;
    if (event.shiftKey) modifiers |= 1;
    if (event.ctrlKey) modifiers |= 2;
    if (event.altKey) modifiers |= 4;
    if (event.metaKey) modifiers |= 8;
    return modifiers;
}
```

**Phase 2: Touch Events (optional, for touch devices)**

```javascript
// Touch devices often don't have mouse events; use touchstart/touchmove/touchend instead
canvas.addEventListener('touchstart', (event) => {
    event.preventDefault();  // Prevent default touch behavior (zoom, scroll)
    
    for (const touch of event.touches) {
        const at = transform_client_to_logical(touch.clientX, touch.clientY);
        const modifiers = get_modifiers(event);
        
        events.push({
            type: 'pointer',
            at,
            button: 0,  // Touch is always "left-button"
            pressed: true,
            released: false,
            moved: false,
            modifiers
        });
    }
});

canvas.addEventListener('touchmove', (event) => {
    event.preventDefault();
    
    for (const touch of event.touches) {
        const at = transform_client_to_logical(touch.clientX, touch.clientY);
        const modifiers = get_modifiers(event);
        
        if (at !== last_touch_pos[touch.identifier]) {
            events.push({
                type: 'pointer',
                at,
                button: 0,
                pressed: false,
                released: false,
                moved: true,
                modifiers
            });
            last_touch_pos[touch.identifier] = at;
        }
    }
});

canvas.addEventListener('touchend', (event) => {
    event.preventDefault();
    
    for (const touch of event.changedTouches) {
        const at = transform_client_to_logical(touch.clientX, touch.clientY);
        const modifiers = get_modifiers(event);
        
        events.push({
            type: 'pointer',
            at,
            button: 0,
            pressed: false,
            released: true,
            moved: false,
            modifiers
        });
        delete last_touch_pos[touch.identifier];
    }
});
```

### 2. Keyboard Events

#### Source: `keydown`, `keyup`

**DOM KeyboardEvent properties**:
- `key` — character pressed (e.g., "a", "Enter", "Shift", "ArrowUp")
- `code` — physical key position (e.g., "KeyA", "Enter", "ShiftLeft", "ArrowUp")
- `ctrlKey`, `shiftKey`, `altKey`, `metaKey` — modifier state
- `repeat` — true if key is held and repeating

**Rui Event Type**:
```rust
pub enum Event {
    Key {
        key: Key,      // Semantic (e.g., Key::Enter, Key::Character('a'))
        code: KeyCode, // Physical (e.g., KeyCode::KeyA)
        pressed: bool,
        modifiers: u8, // shift=1, control=2, alt=4, meta=8
    },
    // ... other variants
}

pub enum Key {
    Character(char),
    Enter,
    Tab,
    Escape,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    // ... etc
}

pub enum KeyCode {
    KeyA,  // physical position, not character
    KeyB,
    // ... etc
}
```

#### Implementation

**Phase 1: Basic Keyboard Events**

```javascript
document.addEventListener('keydown', (event) => {
    // Skip repeat events (already handled by keypress simulation)
    if (event.repeat) return;
    
    const key = map_dom_key_to_rui_key(event.key);
    const code = map_dom_code_to_rui_code(event.code);
    const modifiers = get_modifiers(event);
    
    events.push({
        type: 'key',
        key,
        code,
        pressed: true,
        modifiers
    });
});

document.addEventListener('keyup', (event) => {
    const key = map_dom_key_to_rui_key(event.key);
    const code = map_dom_code_to_rui_code(event.code);
    const modifiers = get_modifiers(event);
    
    events.push({
        type: 'key',
        key,
        code,
        pressed: false,
        modifiers
    });
});

// Mapping tables (Rust-style pseudo-code)
function map_dom_key_to_rui_key(dom_key) {
    const mapping = {
        'Enter': Key::Enter,
        'Tab': Key::Tab,
        'Escape': Key::Escape,
        'Backspace': Key::Backspace,
        'Delete': Key::Delete,
        'ArrowUp': Key::ArrowUp,
        'ArrowDown': Key::ArrowDown,
        'ArrowLeft': Key::ArrowLeft,
        'ArrowRight': Key::ArrowRight,
        'Home': Key::Home,
        'End': Key::End,
        'PageUp': Key::PageUp,
        'PageDown': Key::PageDown,
        ' ': Key::Character(' '),
        // For single-character keys, use the character itself
    };
    
    if (dom_key in mapping) {
        return mapping[dom_key];
    } else if (dom_key.length === 1) {
        return Key::Character(dom_key);  // e.g., 'a', '1', '!', etc.
    } else {
        return Key::Unknown;  // Fallback for unmapped keys
    }
}

function map_dom_code_to_rui_code(dom_code) {
    const mapping = {
        'KeyA': KeyCode::KeyA,
        'KeyB': KeyCode::KeyB,
        // ... etc for all keys
    };
    return mapping[dom_code] || KeyCode::Unknown;
}
```

**Phase 2: Keyboard Repeat (optional)**

Browser automatically fires `keydown` events repeatedly while a key is held. This is correct for rui (handlers receive repeated press events). No special handling needed.

**Phase 3: Composition Input (for IME)**

Text input methods (Chinese, Japanese, Korean IME) use composition events. See Composition Events section below.

### 3. Wheel Events

#### Source: `wheel`

**DOM WheelEvent properties**:
- `deltaX`, `deltaY`, `deltaZ` — scroll amount (negative = scroll left/up, positive = scroll right/down)
- `deltaMode` — 0=pixels, 1=lines, 2=pages
- `ctrlKey`, `shiftKey`, `altKey`, `metaKey` — modifier state

**Rui Event Type**:
```rust
pub enum Event {
    Scroll {
        delta: Point,  // (deltaX, deltaY) in logical units
        modifiers: u8,
    },
    // ... other variants
}
```

#### Implementation

```javascript
canvas.addEventListener('wheel', (event) => {
    event.preventDefault();  // Prevent browser default scroll
    
    let delta_x = event.deltaX;
    let delta_y = event.deltaY;
    
    // Normalize to pixels if needed
    if (event.deltaMode === 1) {  // Lines
        delta_x *= 16;  // ~16 pixels per line
        delta_y *= 16;
    } else if (event.deltaMode === 2) {  // Pages
        delta_x *= canvas.height;
        delta_y *= canvas.height;
    }
    
    // Convert to logical units
    delta_x /= scale_factor;
    delta_y /= scale_factor;
    
    // Invert if needed (different browsers have different sign conventions)
    // Typically: negative = scroll up/left (zoom in), positive = scroll down/right
    
    const modifiers = get_modifiers(event);
    
    events.push({
        type: 'scroll',
        delta: { x: delta_x, y: delta_y },
        modifiers
    });
});
```

### 4. Composition Input (IME)

#### Source: `compositionstart`, `compositionupdate`, `compositionend`

Text input methods fire composition events when typing in CJK or other complex scripts.

**DOM CompositionEvent properties**:
- `data` — partial or completed composition string
- `type` — 'compositionstart', 'compositionupdate', 'compositionend'

**Rui Event Type**:
```rust
pub enum Event {
    Composition {
        text: String,
        finished: bool,  // true on compositionend, false on compositionstart/update
    },
    // ... other variants
}
```

#### Implementation

```javascript
let composition_text = '';

document.addEventListener('compositionstart', (event) => {
    composition_text = event.data || '';
    events.push({
        type: 'composition',
        text: composition_text,
        finished: false
    });
});

document.addEventListener('compositionupdate', (event) => {
    composition_text = event.data || '';
    events.push({
        type: 'composition',
        text: composition_text,
        finished: false
    });
});

document.addEventListener('compositionend', (event) => {
    composition_text = event.data || '';
    events.push({
        type: 'composition',
        text: composition_text,
        finished: true
    });
    composition_text = '';
});
```

### 5. Focus Events

#### Source: `focus`, `blur`

Browser fires these when canvas gains/loses keyboard focus.

**Rui Event Type**:
```rust
pub enum Event {
    Focus { gained: bool },  // true on focus, false on blur
    // ... other variants
}
```

#### Implementation

```javascript
canvas.addEventListener('focus', () => {
    events.push({ type: 'focus', gained: true });
});

canvas.addEventListener('blur', () => {
    events.push({ type: 'focus', gained: false });
});
```

### 6. Resize Events

#### Source: `resize` on window

Browser fires when window is resized.

**Rui Event Type**:
```rust
pub enum Event {
    Resize { width: u32, height: u32, scale_factor: f32 },
    // ... other variants
}
```

#### Implementation

```javascript
window.addEventListener('resize', () => {
    // Update canvas size
    const rect = canvas.getBoundingClientRect();
    const scale_factor = window.devicePixelRatio;
    const width = Math.round(rect.width);
    const height = Math.round(rect.height);
    
    canvas.width = width * scale_factor;
    canvas.height = height * scale_factor;
    
    events.push({
        type: 'resize',
        width,
        height,
        scale_factor
    });
});
```

## Implementation Checklist

### Phase 1: Foundation

- [ ] Implement `Backend::pump()` event loop:
  - [ ] Collect DOM events (mousedown, mouseup, mousemove, keydown, keyup)
  - [ ] Transform to rui Event types
  - [ ] Return `Vec<Event>` to frame loop
- [ ] Implement coordinate transformation (see COORDINATE_CONTRACT.md)
- [ ] Implement modifier extraction (shift, ctrl, alt, meta)
- [ ] Basic test: click at (0, 0) → handler receives pointer at logical (0, 0)

### Phase 2: Enhancement

- [ ] Add touch event support (touchstart, touchmove, touchend)
- [ ] Add wheel/scroll events
- [ ] Add composition input (for IME)
- [ ] Add window resize events
- [ ] Test: keyboard typing in text field
- [ ] Test: scroll wheel in list
- [ ] Test: on non-English keyboard layout

### Phase 3: Integration

- [ ] Verify event translation is transparent to upper layers:
  - [ ] `grep -n "clientX\|clientY\|KeyboardEvent" src/` — should be in platform/wasm.rs only
- [ ] Verify handlers receive only logical coordinates and semantic keys
- [ ] Parity test: same UI action (click, type, scroll) produces same handler call on WASM vs native
- [ ] Run `cargo test --test wasm_event_parity`

## Testing Strategy

### Unit Tests (in platform/wasm.rs or tests/wasm_events.rs)

```rust
#[test]
fn dom_pointer_to_rui_pointer() {
    let dom_event = MouseEvent {
        clientX: 960.0,
        clientY: 540.0,
        button: 0,
        ctrlKey: false,
        shiftKey: true,
        // ...
    };
    
    let rui_event = translate_pointer_event(dom_event);
    
    assert_eq!(rui_event.at.x, 480.0);  // Logical coords (Retina: device/2)
    assert_eq!(rui_event.at.y, 270.0);
    assert_eq!(rui_event.button, 0);
    assert_eq!(rui_event.modifiers, 1);  // shift bit set
}

#[test]
fn dom_key_to_rui_key() {
    let dom_event = KeyboardEvent {
        key: 'a',
        code: 'KeyA',
        ctrlKey: true,
        // ...
    };
    
    let rui_event = translate_key_event(dom_event);
    
    assert_eq!(rui_event.key, Key::Character('a'));
    assert_eq!(rui_event.code, KeyCode::KeyA);
    assert_eq!(rui_event.modifiers, 2);  // control bit set
}
```

### Integration Tests (in tests/wasm_integration.rs)

```rust
#[test]
fn wasm_button_click_fires_handler() {
    let mut h = Harness::new(App { count: 0 }, view)
        .on_backend(Backend::Wasm);
    
    h.click_at(Point { x: 100.0, y: 100.0 });  // Logical coords
    
    assert_eq!(h.state().count, 1);
}

#[test]
fn wasm_text_input_updates_field() {
    let mut h = Harness::new(App { text: String::new() }, view)
        .on_backend(Backend::Wasm);
    
    h.focus_text_field("input");
    h.type_text("hello");
    
    assert_eq!(h.state().text, "hello");
}
```

### Parity Tests (in tests/wasm_parity.rs)

```rust
#[test]
fn wasm_and_x11_pointer_events_equivalent() {
    // Same click at same logical position should produce identical results
    let mut wasm_harness = Harness::new(state.clone(), view).on_backend(Backend::Wasm);
    let mut x11_harness = Harness::new(state.clone(), view).on_backend(Backend::X11);
    
    wasm_harness.click_at(Point { x: 200.0, y: 300.0 });
    x11_harness.click_at(Point { x: 200.0, y: 300.0 });
    
    assert_eq!(wasm_harness.state(), x11_harness.state());
}
```

## Common Pitfalls

### Pitfall 1: Forgetting to Transform clientX/Y

**Wrong**:
```javascript
// clientX is in viewport coords, not canvas coords!
const pointer_x = event.clientX / scale_factor;
```

**Right**:
```javascript
// Step 1: viewport → canvas coords
const canvas_x = event.clientX * (canvas.width / window.innerWidth);
// Step 2: canvas coords → logical coords
const pointer_x = canvas_x / scale_factor;
```

### Pitfall 2: Mixing Key and KeyCode

**Wrong**:
```javascript
// 'key' is semantic ("a"), 'code' is physical ("KeyA")
const key = event.code;  // Should use event.key!
```

**Right**:
```javascript
const semantic_key = event.key;  // For typing and text input
const physical_key = event.code; // For game controls and shortcuts
```

### Pitfall 3: Not Preventing Default Browser Behavior

**Wrong**:
```javascript
canvas.addEventListener('wheel', (event) => {
    // Browser now scrolls the page AND fires scroll event handler!
    // Double work, confusing interaction
});
```

**Right**:
```javascript
canvas.addEventListener('wheel', (event) => {
    event.preventDefault();  // Stop browser default scroll
    // Now only app scroll handler fires
});
```

### Pitfall 4: Using event.buttons Instead of event.button

**Wrong**:
```javascript
// event.buttons is a bitmask of currently pressed buttons (for drag)
// event.button is which button triggered this specific event
const button = event.buttons;  // Wrong for determining what button fired
```

**Right**:
```javascript
const button = event.button;  // 0=left, 1=middle, 2=right
const all_pressed = event.buttons;  // Bitmask for multi-button drag
```

---

**Previous document**: STEP_4_RECIPE_1_COORDINATE_CONTRACT.md — Coordinate transformation rules  
**Part of**: STEP 4: Extract Recipe 1 WASM Documentation  
**Related**: STEP_4_RECIPE_1_ANALYSIS.md, STEP_4_RECIPE_1_VERIFICATION_GATES.md
