# Accessibility Best Practices for rui-native

Building accessible user interfaces ensures everyone can use your app.

## 1. Color & Contrast

**✅ DO:** Use semantic colors
```rust
let text_color = Tone::Text;        // Auto theme support
let accent_color = Tone::Accent;    // Always accessible
```

**❌ DON'T:** Hard-code colors
```rust
fill_color = Color::rgb(0xc0, 0xc0, 0xc0)  // Breaks theme
```

## 2. Keyboard Navigation

**✅ DO:** Support all keyboard interactions
```rust
widgets::button("Submit", |app| app.submit())  // Works with keyboard
```

**❌ DON'T:** Require mouse
```rust
.on_drag(|app, drag| {...})  // Add keyboard alternative!
```

## 3. Focus Management

**✅ DO:** Preserve tab order
```rust
col((
    text_field("First"),   // Focused 1st
    text_field("Second"),  // Focused 2nd
    text_field("Third"),   // Focused 3rd
))
```

## 4. Clear Error Messages

**✅ DO:** Be specific
```rust
text("Email must contain @ symbol")
text("Password must be at least 8 characters")
```

**❌ DON'T:** Use cryptic errors
```rust
text("ERR_PARSE_101")
```

## 5. Input Methods

Your app automatically supports:
- ✅ Keyboard (Tab, Enter, arrows)
- ✅ Mouse (click, drag)
- ✅ Touch (pointer events)
- ✅ Screen readers (platform integration)

## 6. Testing

### Keyboard-Only Test
1. Disconnect mouse
2. Navigate with Tab/Shift+Tab
3. Activate buttons with Enter
4. Use arrows for lists

Every feature should work without a mouse.

### Screen Reader Test
- **macOS:** VoiceOver (Cmd+F5)
- **Windows:** Narrator (Windows+Enter)
- **Linux:** Orca (Alt+Super+O)

### Color Contrast
Use [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- Minimum: 4.5:1 (WCAG AA)
- Better: 7:1 (WCAG AAA)

## 7. High-Contrast Themes

Use these for maximum accessibility:
```rust
Appearance::HighContrastLight  // Black on white (7:1+)
Appearance::HighContrastDark   // White on black (7:1+)
```

## 8. Quick Checklist

- [ ] Tab order matches reading order
- [ ] Focus is always visible
- [ ] All inputs are labeled
- [ ] Error messages are clear
- [ ] Color contrast is 4.5:1+
- [ ] Keyboard works for everything
- [ ] High-contrast mode is supported

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [WebAIM](https://webaim.org/)
- [A11y Project](https://www.a11yproject.com/)

---

**Accessibility is a right, not a feature. Build for everyone.**
