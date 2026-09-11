//! Cross-platform system tray icon with menu support.
//!
//! Provides a tray icon that lives independently of the main window,
//! allowing an application to continue running after its window closes
//! and providing quick access to app functions via a menu.
//!
//! # Design Rationale
//!
//! ## Lifecycle Independence
//! The tray icon is not owned by or tied to the window lifecycle. It is created
//! explicitly by the application and dropped when no longer needed. This allows
//! an app to keep running even after the main window closes—useful for system
//! utilities, daemons, and background services.
//!
//! ## Event Model
//! Events are posted to a thread-safe queue by platform-specific handlers.
//! The app drains this queue once per frame on its own thread, avoiding
//! callback threads and unpredictable execution contexts. This mirrors rui's
//! event model for the window itself: describe once, handle once per frame.
//!
//! ## Menu Reconstruction
//! Menu items are rebuilt every frame from application state, not retained.
//! There are no callbacks or closures in Tray; the app supplies new menu
//! items each frame and matches on event IDs to take action. This keeps the
//! tray stateless and reactive.
//!
//! ## No Platform Callbacks
//! Platform implementations use interior mutability (RefCell, Mutex) to post
//! events without holding a mutable reference to Tray. All unsafe code is
//! isolated to `src/tray/platform/` files; the public API is safe.
//!
//! # Usage
//!
//! ```ignore
//! use rui::tray::{Tray, TrayMenuItem};
//!
//! let icon_data = include_bytes!("../assets/tray-icon.png");
//! let tray = Tray::new(icon_data, "My App")?;
//!
//! loop {
//!     // Update menu each frame from current app state
//!     let items = vec![
//!         TrayMenuItem {
//!             id: 0,
//!             label: "Open".to_string(),
//!             enabled: true,
//!             selected: false,
//!         },
//!         TrayMenuItem {
//!             id: 1,
//!             label: "Quit".to_string(),
//!             enabled: true,
//!             selected: false,
//!         },
//!     ];
//!     tray.set_menu(items)?;
//!
//!     // Drain events once per frame, on the app's thread
//!     for event in tray.drain_events() {
//!         match event {
//!             rui::tray::TrayEvent::MenuItemClicked(id) => {
//!                 if id == 1 {
//!                     break; // Quit
//!                 }
//!             }
//!             rui::tray::TrayEvent::IconActivated => {
//!                 // Show window, or other action
//!             }
//!         }
//!     }
//! }
//! ```

use std::sync::{Arc, Mutex};

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(all(target_os = "linux", feature = "linux-tray"))]
use linux::TrayInner;
#[cfg(target_os = "macos")]
use macos::TrayInner;
#[cfg(target_os = "windows")]
use windows::TrayInner;
#[cfg(all(target_os = "linux", not(feature = "linux-tray")))]
struct TrayInner;

#[cfg(all(target_os = "linux", not(feature = "linux-tray")))]
impl TrayInner {
    pub fn new(
        _icon_data: &[u8],
        _tooltip: &str,
        _event_queue: Arc<Mutex<Vec<TrayEvent>>>,
        _use_panel: bool,
    ) -> Result<Self, crate::Error> {
        Err(crate::Error::Platform(
            "Tray requires the 'linux-tray' feature on Linux. \
             Add it to Cargo.toml: rui = { version = \"...\", features = [\"linux-tray\"] }"
                .into(),
        ))
    }

    pub fn set_menu(&self, _items: Vec<TrayMenuItem>) -> Result<(), crate::Error> {
        Err(crate::Error::Platform(
            "Tray not available without 'linux-tray' feature".into(),
        ))
    }

    pub fn set_tooltip(&self, _text: &str) -> Result<(), crate::Error> {
        Err(crate::Error::Platform(
            "Tray not available without 'linux-tray' feature".into(),
        ))
    }

    pub fn set_icon(&self, _data: &[u8]) -> Result<(), crate::Error> {
        Err(crate::Error::Platform(
            "Tray not available without 'linux-tray' feature".into(),
        ))
    }

    pub fn set_panel_mode(&self, _enabled: bool) -> Result<(), crate::Error> {
        Err(crate::Error::Platform(
            "Tray not available without 'linux-tray' feature".into(),
        ))
    }
}

/// An event from the system tray.
///
/// Tray events are posted to a thread-safe queue by platform-specific handlers
/// and drained by the application once per frame. This design ensures all event
/// handling happens on the app thread, avoiding callback threads and race conditions.
#[derive(Debug, Clone)]
pub enum TrayEvent {
    /// A menu item was clicked; the contained value is the item's id.
    ///
    /// The id corresponds to the `TrayMenuItem::id` field from the menu
    /// set in the most recent `set_menu()` call. The app matches on this
    /// value to take action, similar to how button clicks are routed in rui.
    MenuItemClicked(usize),
    /// The tray icon itself was activated (clicked on non-menu area).
    ///
    /// Typically used to show the window, toggle visibility, or perform
    /// a primary action. The interpretation is application-dependent.
    IconActivated,
    /// Panel mode: icon clicked, panel should be shown at this screen point.
    ///
    /// Only emitted when the tray is in panel mode (created with `new_with_panel()`
    /// or `set_panel_mode(true)`). The contained position is the status item's
    /// frame origin in screen coordinates, allowing the application to position
    /// a panel window just below the tray icon.
    IconActivatedForPanel {
        /// The screen-coordinate position of the tray icon's frame origin (x, y).
        screen_position: (f64, f64),
    },
}

/// A menu item in the tray icon's menu.
///
/// Menu items are described fresh each frame from application state,
/// not retained by the tray icon. This reactive model keeps the tray
/// stateless: if a menu item should be disabled or have a checkmark,
/// the app supplies a new `TrayMenuItem` with `enabled: false` or
/// `selected: true` on the next call to `set_menu()`.
#[derive(Debug, Clone)]
pub struct TrayMenuItem {
    /// Unique identifier for this item; used in click events.
    ///
    /// When this menu item is clicked, a `TrayEvent::MenuItemClicked(id)`
    /// is posted. The id should match nothing else in the current menu
    /// to avoid ambiguity; the app supplies these ids and is responsible
    /// for their uniqueness.
    pub id: usize,
    /// Display text for this menu item.
    pub label: String,
    /// Whether this item is clickable.
    ///
    /// When `false`, the item is grayed out and clicks are ignored.
    /// The platform may not fire a click event at all for disabled items.
    pub enabled: bool,
    /// Whether this item should display as selected (e.g., with a checkmark).
    ///
    /// On most platforms this is rendered as a checkmark or bullet.
    /// It is purely visual; there is no connection between `selected`
    /// and whether the item is `enabled`.
    pub selected: bool,
}

/// System tray icon with menu support.
///
/// Creates and manages a system tray icon that lives independently of the
/// main application window. The tray icon can display a menu and generate
/// events when clicked.
///
/// # Lifecycle
///
/// The tray is created via `Tray::new()` and lives until dropped. It is not
/// tied to any window and can outlive the main window—the application may
/// close its window while keeping the tray alive and active. This allows
/// for system utilities that run in the background.
///
/// # Event Queue
///
/// Events from platform handlers are posted to an `Arc<Mutex<Vec<TrayEvent>>>`
/// that is shared with all platform implementations. This queue is drained
/// once per frame via `drain_events()`, ensuring all handler calls result
/// in a vec of events ready to process synchronously on the app thread.
///
/// # Panel Mode
///
/// By default, clicking the tray icon shows a standard system menu (NSMenu on macOS).
/// In panel mode (enabled with `new_with_panel()` or `set_panel_mode(true)`),
/// clicking the icon fires `IconActivatedForPanel` events instead, allowing the
/// application to display a custom panel window.
pub struct Tray {
    inner: TrayInner,
    event_queue: Arc<Mutex<Vec<TrayEvent>>>,
    use_panel: bool,
}

impl Tray {
    /// Create a new tray icon.
    ///
    /// # Arguments
    /// - `icon_data`: PNG bytes for the icon. On macOS this should be a
    ///   template image (black with alpha). On Windows, a standard RGB image.
    ///   On Linux with the `linux-tray` feature, a standard RGB PNG.
    /// - `tooltip`: Hover text; displayed differently on each platform.
    ///   On macOS, may show in the menu bar; on Windows, in a tooltip;
    ///   on Linux, as a property.
    ///
    /// # Errors
    /// Returns an error if the tray icon cannot be created, e.g. if the
    /// platform does not support tray icons or the icon data is invalid.
    /// On unsupported platforms (e.g. Linux without `linux-tray` feature),
    /// this returns a "Platform not supported" error.
    ///
    /// # Design Note
    /// The tray icon is created once and lives until dropped. Event handling
    /// is asynchronous: platform handlers post events to a shared queue,
    /// which the app drains once per frame. This avoids callback threads.
    pub fn new(icon_data: &[u8], tooltip: &str) -> Result<Self, crate::Error> {
        let event_queue = Arc::new(Mutex::new(Vec::new()));
        let inner = TrayInner::new(icon_data, tooltip, event_queue.clone(), false)?;
        Ok(Tray {
            inner,
            event_queue,
            use_panel: false,
        })
    }

    /// Create a tray icon that displays a panel instead of a menu when clicked.
    ///
    /// In panel mode, clicking the tray icon fires `IconActivatedForPanel` events
    /// with the status item's screen position, allowing the application to display
    /// a custom panel window. This is useful for apps that want more control over
    /// the dropdown appearance and behavior.
    ///
    /// # Arguments
    /// - `icon_data`: PNG bytes for the icon (same as `new()`)
    /// - `tooltip`: Hover text (same as `new()`)
    ///
    /// # Design Note
    /// Panel mode is optional and does not affect menu items set via `set_menu()`.
    /// The application can still use menu items for reference or fallback,
    /// though in panel mode they will not be displayed by the tray itself.
    pub fn new_with_panel(icon_data: &[u8], tooltip: &str) -> Result<Self, crate::Error> {
        let event_queue = Arc::new(Mutex::new(Vec::new()));
        let inner = TrayInner::new(icon_data, tooltip, event_queue.clone(), true)?;
        Ok(Tray {
            inner,
            event_queue,
            use_panel: true,
        })
    }

    /// Enable or disable panel mode.
    ///
    /// When enabled, clicking the tray icon fires `IconActivatedForPanel` events
    /// instead of showing the standard system menu. When disabled, the standard
    /// menu behavior is restored.
    pub fn set_panel_mode(&mut self, enabled: bool) -> Result<(), crate::Error> {
        self.use_panel = enabled;
        self.inner.set_panel_mode(enabled)?;
        Ok(())
    }

    /// Update the menu items displayed when the tray is right-clicked.
    ///
    /// Call this each frame to reflect the current state of the application.
    /// The menu is not retained by the tray; you rebuild it each frame with
    /// the current set of items. This reactive model avoids a retained tree
    /// that can diverge from app state.
    ///
    /// # Design Note
    /// There are no callbacks or closures in the tray API. The app supplies
    /// fresh menu items each frame, and when an item is clicked, a
    /// `TrayEvent::MenuItemClicked` with the item's id is posted to the
    /// event queue. The app matches on the id to take action—like routing
    /// a button click in rui.
    pub fn set_menu(&self, items: Vec<TrayMenuItem>) -> Result<(), crate::Error> {
        self.inner.set_menu(items)
    }

    /// Change the tooltip/title text.
    ///
    /// Updates the text shown when hovering over the tray icon (on platforms
    /// that support it; display behavior varies). Can be called at any time.
    pub fn set_tooltip(&self, text: &str) -> Result<(), crate::Error> {
        self.inner.set_tooltip(text)
    }

    /// Change the icon image at runtime.
    ///
    /// Supplies new icon data (PNG format). Can be called to animate the icon
    /// or show different states (e.g., connected vs. disconnected).
    pub fn set_icon(&self, data: &[u8]) -> Result<(), crate::Error> {
        self.inner.set_icon(data)
    }

    /// Drain pending events from the tray icon.
    ///
    /// Call once per frame to retrieve and clear all pending tray events.
    /// Events are safe to lose if not drained (the queue is cleared on each
    /// call), but typically you process them synchronously on the app thread
    /// before the next frame.
    ///
    /// # Design Note
    /// This mirrors the rui event model: events are posted by platform
    /// handlers to a thread-safe queue, and the app drains them once per
    /// frame. All handlers run on the app thread, never in a callback.
    pub fn drain_events(&self) -> Vec<TrayEvent> {
        self.event_queue
            .lock()
            .map(|mut queue| queue.drain(..).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A menu item can be created with the required fields.
    #[test]
    fn tray_menu_item_creation() {
        let item = TrayMenuItem {
            id: 1,
            label: "Test".to_string(),
            enabled: true,
            selected: false,
        };

        assert_eq!(item.id, 1);
        assert_eq!(item.label, "Test");
        assert!(item.enabled);
        assert!(!item.selected);
    }

    /// Multiple menu items can coexist with different IDs.
    #[test]
    fn tray_menu_item_unique_ids() {
        let items = [
            TrayMenuItem {
                id: 1,
                label: "First".to_string(),
                enabled: true,
                selected: false,
            },
            TrayMenuItem {
                id: 2,
                label: "Second".to_string(),
                enabled: false,
                selected: true,
            },
            TrayMenuItem {
                id: 3,
                label: "Third".to_string(),
                enabled: true,
                selected: false,
            },
        ];

        assert_eq!(items.len(), 3);
        assert_eq!(items[0].id, 1);
        assert_eq!(items[1].id, 2);
        assert_eq!(items[2].id, 3);
    }

    /// TrayEvent::MenuItemClicked carries the clicked item's ID.
    #[test]
    fn tray_event_menu_item_clicked() {
        let event = TrayEvent::MenuItemClicked(42);
        match event {
            TrayEvent::MenuItemClicked(id) => assert_eq!(id, 42),
            _ => panic!("Expected MenuItemClicked"),
        }
    }

    /// TrayEvent::IconActivated is distinct from MenuItemClicked.
    #[test]
    fn tray_event_icon_activated() {
        let event = TrayEvent::IconActivated;
        match event {
            TrayEvent::IconActivated => {
                // This is correct
            }
            _ => panic!("Expected IconActivated"),
        }
    }

    /// TrayEvent::IconActivatedForPanel carries screen position data.
    #[test]
    fn tray_event_icon_activated_for_panel() {
        let event = TrayEvent::IconActivatedForPanel {
            screen_position: (100.0, 50.0),
        };
        match event {
            TrayEvent::IconActivatedForPanel {
                screen_position: (x, y),
            } => {
                assert_eq!(x, 100.0);
                assert_eq!(y, 50.0);
            }
            _ => panic!("Expected IconActivatedForPanel"),
        }
    }

    /// TrayEvent variants can be cloned and compared.
    #[test]
    fn tray_event_is_cloneable() {
        let event1 = TrayEvent::MenuItemClicked(1);
        let event2 = event1.clone();

        match (event1, event2) {
            (TrayEvent::MenuItemClicked(a), TrayEvent::MenuItemClicked(b)) => {
                assert_eq!(a, b);
            }
            _ => panic!("Clone failed or events don't match"),
        }
    }

    /// TrayMenuItem is cloneable and copies all fields correctly.
    #[test]
    fn tray_menu_item_is_cloneable() {
        let item1 = TrayMenuItem {
            id: 5,
            label: "Clone Test".to_string(),
            enabled: false,
            selected: true,
        };

        let item2 = item1.clone();

        assert_eq!(item1.id, item2.id);
        assert_eq!(item1.label, item2.label);
        assert_eq!(item1.enabled, item2.enabled);
        assert_eq!(item1.selected, item2.selected);
    }

    /// Event queue can be created and is initially empty.
    #[test]
    fn event_queue_starts_empty() {
        let event_queue: Arc<Mutex<Vec<TrayEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let queue = event_queue.lock().unwrap();
        assert!(queue.is_empty());
    }

    /// Multiple threads can access the event queue concurrently.
    #[test]
    fn event_queue_thread_safe() {
        use std::thread;

        let event_queue: Arc<Mutex<Vec<TrayEvent>>> = Arc::new(Mutex::new(Vec::new()));

        let q1 = event_queue.clone();
        let handle1 = thread::spawn(move || {
            if let Ok(mut q) = q1.lock() {
                q.push(TrayEvent::MenuItemClicked(1));
            }
        });

        let q2 = event_queue.clone();
        let handle2 = thread::spawn(move || {
            if let Ok(mut q) = q2.lock() {
                q.push(TrayEvent::IconActivated);
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        let queue = event_queue.lock().unwrap();
        assert_eq!(queue.len(), 2);
    }
}
