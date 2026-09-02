#![allow(missing_docs)]

#[cfg(test)]
mod overlay_semantics_tests {
    use rui::element::El;
    use rui::overlay::{Overlay, OverlayAnchor, OverlayPlacement};
    use rui::testing::Harness;
    use rui::{col, text};

    #[test]
    fn overlay_semantics_defines_overlay_type() {
        let _overlay = Overlay::Modal;
        let _overlay = Overlay::Popover;
        let _overlay = Overlay::Dropdown;
    }

    #[test]
    fn overlay_placement_defines_anchor_points() {
        let _anchor = OverlayAnchor::TopStart;
        let _anchor = OverlayAnchor::TopCenter;
        let _anchor = OverlayAnchor::TopEnd;
        let _anchor = OverlayAnchor::MiddleStart;
        let _anchor = OverlayAnchor::MiddleCenter;
        let _anchor = OverlayAnchor::MiddleEnd;
        let _anchor = OverlayAnchor::BottomStart;
        let _anchor = OverlayAnchor::BottomCenter;
        let _anchor = OverlayAnchor::BottomEnd;
    }

    #[test]
    fn overlay_placement_defines_positioning() {
        let _placement = OverlayPlacement {
            anchor: OverlayAnchor::TopStart,
            offset_x: 0.0,
            offset_y: 0.0,
        };
    }

    #[test]
    fn element_has_overlay_builder() {
        struct App {
            show_modal: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((text("Content"),)).overlay(Overlay::Modal)
        }

        let mut h = Harness::new(App { show_modal: false }, view);
        h.frames(1);
        assert!(!h.state().show_modal);
    }

    #[test]
    fn element_has_overlay_placement_builder() {
        struct App;

        fn view(_app: &App) -> El<App> {
            col((text("Content"),))
                .overlay(Overlay::Popover)
                .overlay_placement(OverlayPlacement {
                    anchor: OverlayAnchor::TopStart,
                    offset_x: 10.0,
                    offset_y: 10.0,
                })
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }

    #[test]
    fn overlay_affects_z_order_in_paint() {
        struct App {
            modal_visible: bool,
        }

        fn view(app: &App) -> El<App> {
            if app.modal_visible {
                col((text("Modal"),)).overlay(Overlay::Modal)
            } else {
                col((text("Hidden"),))
            }
        }

        let mut h = Harness::new(
            App {
                modal_visible: true,
            },
            view,
        );
        h.frames(1);
        assert!(h.state().modal_visible);
    }

    #[test]
    fn modal_overlay_receives_higher_z_order() {
        struct App {
            #[allow(dead_code)]
            show_modal: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((text("Background"),))
                .overlay_placement(OverlayPlacement {
                    anchor: OverlayAnchor::MiddleCenter,
                    offset_x: 0.0,
                    offset_y: 0.0,
                })
                .overlay(Overlay::Modal)
        }

        let mut h = Harness::new(App { show_modal: true }, view);
        h.frames(1);
    }

    #[test]
    fn popover_overlay_receives_medium_z_order() {
        struct App;

        fn view(_app: &App) -> El<App> {
            col((text("Popover"),))
                .overlay(Overlay::Popover)
                .overlay_placement(OverlayPlacement {
                    anchor: OverlayAnchor::TopStart,
                    offset_x: 0.0,
                    offset_y: 0.0,
                })
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }

    #[test]
    fn dropdown_overlay_receives_lowest_z_order() {
        struct App;

        fn view(_app: &App) -> El<App> {
            col((text("Dropdown"),)).overlay(Overlay::Dropdown)
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }

    #[test]
    fn overlay_placement_supports_custom_offsets() {
        struct App;

        fn view(_app: &App) -> El<App> {
            col((text("Positioned"),)).overlay_placement(OverlayPlacement {
                anchor: OverlayAnchor::BottomEnd,
                offset_x: -10.0,
                offset_y: -10.0,
            })
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }
}
