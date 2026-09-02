/// Visual and interaction semantics for overlays (modals, popovers, dropdowns).
///
/// Overlays layer on top of base content with different z-order priorities:
///   - Modal: highest z-order, typically blocks interaction with content below
///   - Popover: medium z-order, often anchored to a trigger element
///   - Dropdown: lowest z-order, typically below other overlays
///
/// Overlay type determines z-order and interaction semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    /// Modal overlay: highest z-order, blocks interaction with content below
    Modal,
    /// Popover overlay: medium z-order, anchored positioning, dismissable
    Popover,
    /// Dropdown overlay: lowest z-order, typically below popovers
    Dropdown,
}

impl Overlay {
    /// Returns the z-order priority for this overlay type.
    /// Higher values render on top.
    pub fn z_order(&self) -> i32 {
        match self {
            Overlay::Modal => 1000,
            Overlay::Popover => 500,
            Overlay::Dropdown => 100,
        }
    }

    /// Returns true if this overlay should block pointer interaction with content below
    pub fn blocks_interaction(&self) -> bool {
        matches!(self, Overlay::Modal)
    }

    /// Returns true if this overlay should be dismissable (typically on Escape or click outside)
    pub fn is_dismissable(&self) -> bool {
        matches!(self, Overlay::Popover | Overlay::Dropdown)
    }
}

/// Anchor point for positioning overlays relative to a trigger element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAnchor {
    /// Top-left corner
    TopStart,
    /// Top-center edge
    TopCenter,
    /// Top-right corner
    TopEnd,
    /// Middle-left edge
    MiddleStart,
    /// Middle-center (absolute center)
    MiddleCenter,
    /// Middle-right edge
    MiddleEnd,
    /// Bottom-left corner
    BottomStart,
    /// Bottom-center edge
    BottomCenter,
    /// Bottom-right corner
    BottomEnd,
}

impl OverlayAnchor {
    /// Returns the relative position (0.0-1.0) within a rectangle for this anchor.
    /// (0.0, 0.0) = top-left, (1.0, 1.0) = bottom-right, (0.5, 0.5) = center
    pub fn relative_position(&self) -> (f32, f32) {
        match self {
            OverlayAnchor::TopStart => (0.0, 0.0),
            OverlayAnchor::TopCenter => (0.5, 0.0),
            OverlayAnchor::TopEnd => (1.0, 0.0),
            OverlayAnchor::MiddleStart => (0.0, 0.5),
            OverlayAnchor::MiddleCenter => (0.5, 0.5),
            OverlayAnchor::MiddleEnd => (1.0, 0.5),
            OverlayAnchor::BottomStart => (0.0, 1.0),
            OverlayAnchor::BottomCenter => (0.5, 1.0),
            OverlayAnchor::BottomEnd => (1.0, 1.0),
        }
    }
}

/// Placement configuration for an overlay element.
/// Specifies anchor point and offset from that point.
#[derive(Debug, Clone, Copy)]
pub struct OverlayPlacement {
    /// Anchor point for positioning
    pub anchor: OverlayAnchor,
    /// Horizontal offset in logical pixels (positive = right)
    pub offset_x: f32,
    /// Vertical offset in logical pixels (positive = down)
    pub offset_y: f32,
}

impl OverlayPlacement {
    /// Creates a new overlay placement at the center.
    pub fn center() -> Self {
        Self {
            anchor: OverlayAnchor::MiddleCenter,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    /// Creates a new overlay placement at top-start with optional offset.
    pub fn top_start(offset_x: f32, offset_y: f32) -> Self {
        Self {
            anchor: OverlayAnchor::TopStart,
            offset_x,
            offset_y,
        }
    }

    /// Creates a new overlay placement at top-center with optional offset.
    pub fn top_center(offset_y: f32) -> Self {
        Self {
            anchor: OverlayAnchor::TopCenter,
            offset_x: 0.0,
            offset_y,
        }
    }

    /// Creates a new overlay placement at top-end with optional offset.
    pub fn top_end(offset_x: f32, offset_y: f32) -> Self {
        Self {
            anchor: OverlayAnchor::TopEnd,
            offset_x,
            offset_y,
        }
    }

    /// Creates a new overlay placement at bottom-center with optional offset.
    pub fn bottom_center(offset_y: f32) -> Self {
        Self {
            anchor: OverlayAnchor::BottomCenter,
            offset_x: 0.0,
            offset_y,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_z_order_is_correctly_ordered() {
        assert!(Overlay::Modal.z_order() > Overlay::Popover.z_order());
        assert!(Overlay::Popover.z_order() > Overlay::Dropdown.z_order());
    }

    #[test]
    fn modal_blocks_interaction() {
        assert!(Overlay::Modal.blocks_interaction());
        assert!(!Overlay::Popover.blocks_interaction());
        assert!(!Overlay::Dropdown.blocks_interaction());
    }

    #[test]
    fn overlay_dismissability_is_correct() {
        assert!(!Overlay::Modal.is_dismissable());
        assert!(Overlay::Popover.is_dismissable());
        assert!(Overlay::Dropdown.is_dismissable());
    }

    #[test]
    fn overlay_anchor_relative_positions() {
        assert_eq!(OverlayAnchor::MiddleCenter.relative_position(), (0.5, 0.5));
        assert_eq!(OverlayAnchor::TopStart.relative_position(), (0.0, 0.0));
        assert_eq!(OverlayAnchor::BottomEnd.relative_position(), (1.0, 1.0));
    }

    #[test]
    fn overlay_placement_constructors() {
        let placement = OverlayPlacement::center();
        assert_eq!(placement.anchor, OverlayAnchor::MiddleCenter);
        assert_eq!(placement.offset_x, 0.0);
        assert_eq!(placement.offset_y, 0.0);

        let placement = OverlayPlacement::top_start(10.0, 20.0);
        assert_eq!(placement.anchor, OverlayAnchor::TopStart);
        assert_eq!(placement.offset_x, 10.0);
        assert_eq!(placement.offset_y, 20.0);
    }
}
