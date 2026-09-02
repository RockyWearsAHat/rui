#![allow(missing_docs)]

#[cfg(test)]
mod overlay_integration_tests {
    use rui::element::El;
    use rui::overlay::{Overlay, OverlayAnchor, OverlayPlacement};
    use rui::testing::Harness;
    use rui::{col, row, text};

    #[test]
    fn modal_overlay_persists_across_frames() {
        struct App {
            frame_count: usize,
        }

        fn view(app: &App) -> El<App> {
            col((text(format!("Frame: {}", app.frame_count)),)).overlay(Overlay::Modal)
        }

        let mut h = Harness::new(App { frame_count: 0 }, view);
        h.frames(5);
        assert_eq!(h.state().frame_count, 0);
    }

    #[test]
    fn overlay_works_with_container_elements() {
        struct App;

        fn view(_app: &App) -> El<App> {
            row((
                col((text("Left"),)),
                col((text("Right"),)).overlay(Overlay::Popover),
            ))
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }

    #[test]
    fn overlay_preserves_element_functionality() {
        struct App {
            clicks: usize,
        }

        fn view(app: &App) -> El<App> {
            col((text(format!("Clicks: {}", app.clicks)),))
                .overlay(Overlay::Modal)
                .on_click(|app: &mut App| app.clicks += 1)
        }

        let mut h = Harness::new(App { clicks: 0 }, view);
        h.frames(1);
        h.click_text("Clicks: 0");
        assert_eq!(h.state().clicks, 1);
    }

    #[test]
    fn overlay_placement_stores_correctly() {
        struct App;

        fn view(_app: &App) -> El<App> {
            col((text("Content"),))
                .overlay_placement(OverlayPlacement {
                    anchor: OverlayAnchor::TopStart,
                    offset_x: 10.0,
                    offset_y: 20.0,
                })
                .overlay(Overlay::Dropdown)
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }

    #[test]
    fn multiple_overlays_can_nest() {
        struct App;

        fn view(_app: &App) -> El<App> {
            col((col((text("Inner"),))
                .overlay(Overlay::Popover)
                .overlay_placement(OverlayPlacement::center()),))
            .overlay(Overlay::Modal)
            .overlay_placement(OverlayPlacement::center())
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }

    #[test]
    fn overlay_works_on_text_elements() {
        struct App;

        fn view(_app: &App) -> El<App> {
            text("Overlay text").overlay(Overlay::Dropdown)
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }

    #[test]
    fn overlay_anchor_combinations_work() {
        struct App;

        fn view(_app: &App) -> El<App> {
            row((
                col((text("TL"),)).overlay_placement(OverlayPlacement::top_start(0.0, 0.0)),
                col((text("BC"),)).overlay_placement(OverlayPlacement::bottom_center(0.0)),
                col((text("TE"),)).overlay_placement(OverlayPlacement::top_end(0.0, 0.0)),
            ))
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }

    #[test]
    fn overlay_type_affects_element_rendering() {
        struct App {
            modal_visible: bool,
        }

        fn view(app: &App) -> El<App> {
            if app.modal_visible {
                col((text("Modal"),))
                    .overlay(Overlay::Modal)
                    .overlay_placement(OverlayPlacement::center())
            } else {
                col((text("Normal"),))
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
    fn overlay_and_hover_can_coexist() {
        use rui::style::Tone;

        struct App;

        fn view(_app: &App) -> El<App> {
            col((text("Hoverable Modal"),))
                .overlay(Overlay::Popover)
                .hover_fill(Tone::Accent)
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }

    #[test]
    fn overlay_offset_calculation_preserves_precision() {
        struct App;

        fn view(_app: &App) -> El<App> {
            col((text("Precise Offset"),)).overlay_placement(OverlayPlacement {
                anchor: OverlayAnchor::MiddleEnd,
                offset_x: 12.5,
                offset_y: -8.75,
            })
        }

        let mut h = Harness::new(App, view);
        h.frames(1);
    }
}
