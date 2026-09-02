//! Loading and empty state recipe functions.
//! These provide pre-built, accessible state-specific UI patterns.

use crate::*;

/// Displays an empty state with title, icon, and action prompt.
/// Used when a list, feed, or search has no items.
///
/// # Example
/// ```ignore
/// if items.is_empty() {
///     empty_state("No items", "Create one")
/// } else {
///     list_view(items)
/// }
/// ```
pub fn empty_state<S: 'static>(title: &str, action: &str) -> El<S> {
    col((
        // Icon (represented as a large character)
        text("○").color(Tone::Muted).text_size(48.0),
        // Title
        text(title).color(Tone::Muted),
        // Action prompt
        text(action).color(Tone::Muted).text_size(12.0),
    ))
    .gap(12.0)
    .pad(24.0)
    .fill(Tone::Idle)
    .center()
}

/// Displays a loading state with animated spinner and message.
/// Used when data is being fetched. Never show under 300ms—keep stale data visible.
///
/// # Example
/// ```ignore
/// if fetching {
///     loading_state("Loading items...")
/// } else {
///     list_view(items)
/// }
/// ```
pub fn loading_state<S: 'static>(message: &str) -> El<S> {
    col((
        // Spinner (animated character)
        text("⟳").color(Tone::Muted).text_size(32.0),
        // Message
        text(message).color(Tone::Muted),
    ))
    .gap(16.0)
    .pad(24.0)
    .fill(Tone::Idle)
    .center()
}

/// Displays stale data indicator with refresh action.
/// Used when cached data is older than acceptable.
///
/// # Example
/// ```ignore
/// if data_age_seconds > 300 {
///     stale_data_state("Data from 5 minutes ago")
/// } else {
///     current_view(data)
/// }
/// ```
pub fn stale_data_state<S: 'static>(message: &str) -> El<S> {
    col((
        text("⚠").color(Tone::Warn).text_size(24.0),
        text(message).color(Tone::Warn),
    ))
    .gap(12.0)
    .pad(16.0)
    .fill(Tone::Idle)
    .center()
}

/// Displays error state with retry action.
/// Used when an operation fails and retry is appropriate.
///
/// # Example
/// ```ignore
/// if let Some(error) = app.error {
///     error_state(&error)
/// } else {
///     success_view()
/// }
/// ```
pub fn error_state<S: 'static>(message: &str) -> El<S> {
    col((
        text("✕").color(Tone::Bad).text_size(24.0),
        text(message).color(Tone::Bad),
    ))
    .gap(12.0)
    .pad(16.0)
    .fill(Tone::Idle)
    .center()
}
