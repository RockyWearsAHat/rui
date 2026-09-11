//! Linux system tray implementation using freedesktop StatusNotifierItem.
//!
//! Implements the StatusNotifierItem D-Bus interface for freedesktop-compliant
//! desktop environments (GNOME, KDE, sway, etc.). This is the modern, standards-based
//! replacement for the deprecated KDE system tray (which used a different API).
//!
//! # Design
//!
//! The tray runs a background tokio runtime on a spawned thread that:
//! - Registers as a StatusNotifierItem on the D-Bus session bus
//! - Handles Activate() and ContextMenu() method calls
//! - Posts events to the shared event queue
//! - Listens for menu/tooltip/icon updates via mpsc channel
//!
//! The menu is rebuilt each frame and stored locally. On ContextMenu(),
//! the environment reads the menu properties (not yet implemented; for now,
//! it would be a simple property return).
//!
//! ## Thread Model
//!
//! The background thread runs a full tokio runtime to handle async D-Bus methods
//! and signals. Commands are sent via mpsc from the main app thread. Events are
//! posted to the shared Arc<Mutex<Vec<TrayEvent>>>, which is safe to access
//! concurrently.
//!
//! ## Feature Gate
//!
//! This implementation is behind the `linux-tray` feature to avoid pulling
//! in zbus and tokio on builds that don't need a tray. On Linux without
//! the feature, Tray::new() returns an unsupported error.

use crate::Error;

use super::{TrayEvent, TrayMenuItem};

#[cfg(feature = "linux-tray")]
use std::sync::{mpsc, Arc, Mutex};
#[cfg(feature = "linux-tray")]
use std::thread;

/// Linux tray implementation using StatusNotifierItem D-Bus service.
#[cfg(feature = "linux-tray")]
pub struct TrayInner {
    event_queue: Arc<Mutex<Vec<TrayEvent>>>,
    tx: mpsc::Sender<TrayCommand>,
}

#[cfg(feature = "linux-tray")]
#[derive(Debug, Clone)]
enum TrayCommand {
    SetMenu(Vec<TrayMenuItem>),
    SetTooltip(String),
    SetIcon(Vec<u8>),
}

#[cfg(feature = "linux-tray")]
impl TrayInner {
    /// Create a new tray icon on Linux via StatusNotifierItem D-Bus service.
    pub fn new(
        _icon_data: &[u8],
        _tooltip: &str,
        event_queue: Arc<Mutex<Vec<TrayEvent>>>,
    ) -> Result<Self, Error> {
        let event_queue_clone = event_queue.clone();

        let (tx, rx) = mpsc::channel::<TrayCommand>();

        // Spawn background thread to run D-Bus service
        thread::spawn(move || {
            if let Err(e) = run_dbus_service(event_queue_clone, rx) {
                eprintln!("Tray D-Bus service error: {}", e);
            }
        });

        Ok(TrayInner { event_queue, tx })
    }

    /// Update the menu items by sending a command to the D-Bus thread.
    pub fn set_menu(&self, items: Vec<TrayMenuItem>) -> Result<(), Error> {
        self.tx
            .send(TrayCommand::SetMenu(items))
            .map_err(|_| Error::Platform("Tray service channel closed".into()))?;
        Ok(())
    }

    /// Update the tooltip text.
    pub fn set_tooltip(&self, text: &str) -> Result<(), Error> {
        self.tx
            .send(TrayCommand::SetTooltip(text.to_string()))
            .map_err(|_| Error::Platform("Tray service channel closed".into()))?;
        Ok(())
    }

    /// Update the icon image.
    pub fn set_icon(&self, data: &[u8]) -> Result<(), Error> {
        self.tx
            .send(TrayCommand::SetIcon(data.to_vec()))
            .map_err(|_| Error::Platform("Tray service channel closed".into()))?;
        Ok(())
    }
}

#[cfg(feature = "linux-tray")]
fn run_dbus_service(
    event_queue: Arc<Mutex<Vec<TrayEvent>>>,
    rx: mpsc::Receiver<TrayCommand>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a new tokio runtime for this thread
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async {
        // Connect to the session D-Bus
        let connection = zbus::ConnectionBuilder::session()?.build().await?;

        // Create a unique name for this StatusNotifierItem
        let unique_name = format!(
            "org.kde.StatusNotifierItem-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );

        // Register the object on the session bus
        let path = "/StatusNotifierItem";
        let item = StatusNotifierItem::new(event_queue.clone(), Arc::new(Mutex::new(rx)));
        connection.object_server().at(path, item).await?;

        // Register the object name (must do this before calling RegisterStatusNotifierItem)
        // Use the string slice instead of the owned String
        let unique_name_str = unique_name.as_str();
        connection.request_name(unique_name_str).await?;

        // Register with the StatusNotifierWatcher (best-effort; don't fail if unavailable)
        let _ = register_with_watcher(&connection, unique_name_str, path).await;

        // Keep the connection alive by running the object server
        // The object server handles incoming D-Bus calls automatically
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    })
}

#[cfg(feature = "linux-tray")]
struct StatusNotifierItem {
    event_queue: Arc<Mutex<Vec<TrayEvent>>>,
    rx: Arc<Mutex<mpsc::Receiver<TrayCommand>>>,
    menu_items: Arc<Mutex<Vec<TrayMenuItem>>>,
    tooltip: Arc<Mutex<String>>,
}

#[cfg(feature = "linux-tray")]
impl StatusNotifierItem {
    fn new(
        event_queue: Arc<Mutex<Vec<TrayEvent>>>,
        rx: Arc<Mutex<mpsc::Receiver<TrayCommand>>>,
    ) -> Self {
        StatusNotifierItem {
            event_queue,
            rx,
            menu_items: Arc::new(Mutex::new(Vec::new())),
            tooltip: Arc::new(Mutex::new("Tray Item".to_string())),
        }
    }
}

#[cfg(feature = "linux-tray")]
#[zbus::interface(name = "org.kde.StatusNotifierItem")]
impl StatusNotifierItem {
    /// Called when the status notifier item is activated (clicked).
    async fn activate(&self, _x: i32, _y: i32) -> zbus::fdo::Result<()> {
        if let Ok(mut queue) = self.event_queue.lock() {
            queue.push(TrayEvent::IconActivated);
        }
        Ok(())
    }

    /// Called when context menu is requested (right-click).
    ///
    /// The menu structure is exposed via the ContextMenu property. The watcher/panel
    /// will read it and display the menu to the user. Menu item selections are tracked
    /// via D-Bus signal emissions or by polling the menu state.
    async fn context_menu(&self, _x: i32, _y: i32) -> zbus::fdo::Result<()> {
        // Process any pending commands from the main thread
        if let Ok(mut rx) = self.rx.lock() {
            if let Ok(cmd) = rx.try_recv() {
                match cmd {
                    TrayCommand::SetMenu(items) => {
                        if let Ok(mut menu) = self.menu_items.lock() {
                            *menu = items;
                        }
                    }
                    TrayCommand::SetTooltip(text) => {
                        if let Ok(mut tooltip) = self.tooltip.lock() {
                            *tooltip = text;
                        }
                    }
                    TrayCommand::SetIcon(_data) => {
                        // Icon updates are not currently exposed on Linux D-Bus
                    }
                }
            }
        }
        // The menu is now available via the ContextMenu property
        Ok(())
    }

    /// Called on scroll wheel events.
    async fn scroll(&self, _delta: i32, _orientation: String) -> zbus::fdo::Result<()> {
        Ok(())
    }

    /// Called on secondary activation (middle click).
    async fn secondary_activate(&self, _x: i32, _y: i32) -> zbus::fdo::Result<()> {
        Ok(())
    }

    /// Property: Current status of the item (typically "Active", "Passive", "NeedsAttention").
    #[zbus(property)]
    fn status(&self) -> String {
        "Active".to_string()
    }

    /// Property: Title/label of the item.
    #[zbus(property)]
    fn title(&self) -> String {
        "Tray Item".to_string()
    }

    /// Property: Unique identifier.
    #[zbus(property)]
    fn id(&self) -> String {
        format!("{}", std::process::id())
    }

    /// Property: Tooltip/help text.
    #[zbus(property)]
    fn tooltip_title(&self) -> String {
        self.tooltip
            .lock()
            .map(|t| t.clone())
            .unwrap_or_else(|_| "Item".to_string())
    }

    /// Property: Category (ApplicationStatus, Communications, SystemServices, Hardware).
    #[zbus(property)]
    fn category(&self) -> String {
        "ApplicationStatus".to_string()
    }

    /// Property: Context menu structure (menu items as array of structs).
    /// Returns array of (id, label, enabled, selected) tuples representing menu items.
    #[zbus(property)]
    fn context_menu(&self) -> Vec<(u32, String, bool, bool)> {
        self.menu_items
            .lock()
            .map(|items| {
                items
                    .iter()
                    .map(|item| {
                        (
                            item.id as u32,
                            item.label.clone(),
                            item.enabled,
                            item.selected,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(feature = "linux-tray")]
async fn register_with_watcher(
    connection: &zbus::Connection,
    item_name: &str,
    item_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to register with the status notifier watcher.
    // This notifies the panel/systray implementation that a new status notifier item exists.
    // This is optional; if the watcher is not available, applications can still function
    // by being discovered via other means (e.g., property polling).

    // Try to call org.kde.StatusNotifierWatcher.RegisterStatusNotifierItem
    let watcher_service = "org.kde.StatusNotifierWatcher";
    let watcher_path = "/StatusNotifierWatcher";
    let watcher_interface = "org.kde.StatusNotifierWatcher";

    // Construct the service name as "unique_name + object_path"
    let service_identifier = format!("{}{}", item_name, item_path);

    // Attempt the registration call (ignore failures as watcher may not be running)
    let _ = connection
        .call_method(
            Some(watcher_service),
            watcher_path,
            Some(watcher_interface),
            "RegisterStatusNotifierItem",
            &service_identifier,
        )
        .await;

    Ok(())
}

/// Linux tray stub when the `linux-tray` feature is not enabled.
#[cfg(all(target_os = "linux", not(feature = "linux-tray")))]
pub struct TrayInner;

/// Linux tray stub when the `linux-tray` feature is not enabled.
#[cfg(all(target_os = "linux", not(feature = "linux-tray")))]
impl TrayInner {
    /// Create a new tray icon on Linux without the `linux-tray` feature.
    pub fn new(
        _icon_data: &[u8],
        _tooltip: &str,
        _event_queue: std::sync::Arc<std::sync::Mutex<Vec<TrayEvent>>>,
    ) -> Result<Self, Error> {
        Err(Error::Platform(
            "Tray requires the 'linux-tray' feature on Linux. \
             Add it to Cargo.toml: rui = { version = \"...\", features = [\"linux-tray\"] }"
                .into(),
        ))
    }

    /// Update the menu items (stub).
    pub fn set_menu(&self, _items: Vec<TrayMenuItem>) -> Result<(), Error> {
        Err(Error::Platform(
            "Tray not available without 'linux-tray' feature".into(),
        ))
    }

    /// Update the tooltip text (stub).
    pub fn set_tooltip(&self, _text: &str) -> Result<(), Error> {
        Err(Error::Platform(
            "Tray not available without 'linux-tray' feature".into(),
        ))
    }

    /// Update the icon image (stub).
    pub fn set_icon(&self, _data: &[u8]) -> Result<(), Error> {
        Err(Error::Platform(
            "Tray not available without 'linux-tray' feature".into(),
        ))
    }
}
