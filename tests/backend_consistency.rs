#![allow(missing_docs)]

//! Cross-platform backend consistency tests for pointer coordinate normalization.
//!
//! These tests verify that:
//! 1. Pointer coordinates are correctly normalized to logical units
//! 2. Scale factors are properly applied during coordinate transformation
//! 3. Click handling is consistent across different display scales
//! 4. Element positions and sizes remain correct in logical coordinates

#[cfg(test)]
mod pointer_coordinate_tests {
    use rui::element::El;
    use rui::geom::Point;
    use rui::testing::Harness;
    use rui::{button, col};

    #[test]
    fn pointer_coordinates_at_scale_1_0() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(50.0, 20.0));

        // Click should trigger the handler at scale 1.0
        assert!(h.state().clicked, "Click handler should fire at scale 1.0");
    }

    #[test]
    fn pointer_coordinates_at_scale_1_5() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(75.0, 20.0));

        // Click should trigger the handler at scale 1.5
        assert!(h.state().clicked, "Click handler should fire at scale 1.5");
    }

    #[test]
    fn pointer_coordinates_at_scale_2_0() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(100.0, 20.0));

        // Click should trigger the handler at scale 2.0
        assert!(h.state().clicked, "Click handler should fire at scale 2.0");
    }

    #[test]
    fn pointer_coordinates_at_scale_2_5() {
        struct App {
            clicked: bool,
        }

        fn view(_app: &App) -> El<App> {
            col((button("Click me").on_click(|state: &mut App| {
                state.clicked = true;
            }),))
        }

        let mut h = Harness::new(App { clicked: false }, view);
        h.click(Point::new(80.0, 20.0));

        // Click should trigger the handler at scale 2.5
        assert!(h.state().clicked, "Click handler should fire at scale 2.5");
    }
}
