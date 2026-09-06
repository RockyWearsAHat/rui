//! Tests for the skeleton loading placeholder component.

#[cfg(test)]
mod skeleton_tests {
    use rui::skeleton::{skeleton, skeleton_rows};
    use rui::style::Length;
    use rui::testing::Harness;
    use rui::{col, El};

    /// Verify that skeleton_rows draws the requested number of rows.
    #[test]
    fn skeleton_rows_draw_the_requested_count() {
        #[derive(Default)]
        struct Nothing;

        fn view(_: &Nothing) -> El<Nothing> {
            skeleton_rows(5, 18.0)
        }

        let mut harness = Harness::new(Nothing, view).size(200.0, 200.0);
        harness.frame();

        // Check that all 5 rows exist with the correct keys
        for i in 0..5 {
            let key = format!("skeleton-{}", i);
            assert!(
                harness.find_key(&key).is_some(),
                "skeleton-{} should exist",
                i
            );
        }

        // Verify that row 5 does NOT exist
        assert!(
            harness.find_key("skeleton-5").is_none(),
            "skeleton-5 should not exist when only 5 rows requested"
        );
    }

    /// Verify that skeleton paints inside its own box and not beyond.
    #[test]
    fn skeleton_paints_inside_its_own_box() {
        #[derive(Default)]
        struct Nothing;

        fn view(_: &Nothing) -> El<Nothing> {
            col(skeleton(Length::Fixed(100.0), 20.0))
        }

        let mut harness = Harness::new(Nothing, view).size(200.0, 100.0);
        harness.frame();

        // Skeleton should paint something in its area (100x20)
        let rect_inside = rui::Rect::new(0.0, 0.0, 100.0, 20.0);
        assert!(
            harness.marked(rect_inside),
            "skeleton should paint inside its own rectangle"
        );

        // But nothing outside its bounds
        let rect_outside = rui::Rect::new(100.0, 0.0, 50.0, 20.0);
        assert!(
            !harness.marked(rect_outside),
            "skeleton should not paint beyond its bounds"
        );
    }

    /// Verify that skeleton holds no text.
    #[test]
    fn skeleton_holds_no_text() {
        #[derive(Default)]
        struct Nothing;

        fn view(_: &Nothing) -> El<Nothing> {
            col(skeleton(Length::Fixed(100.0), 20.0))
        }

        let mut harness = Harness::new(Nothing, view).size(200.0, 100.0);
        harness.frame();

        // Text should be empty
        assert!(harness.text().is_empty(), "skeleton should contain no text");
    }

    /// Verify that skeleton is not focusable.
    #[test]
    fn skeleton_is_not_focusable() {
        #[derive(Default)]
        struct Nothing;

        fn view(_: &Nothing) -> El<Nothing> {
            col(skeleton(Length::Fixed(100.0), 20.0))
        }

        let mut harness = Harness::new(Nothing, view).size(200.0, 100.0);
        harness.frame();

        // Try to focus: the skeleton should not accept focus
        let initial_focused = harness.focused();
        harness.key(rui::Key::Tab);
        harness.frame();

        // Since skeleton has no focusable elements, focus should not change
        assert_eq!(
            initial_focused,
            harness.focused(),
            "skeleton should not be focusable"
        );
    }

    /// Verify that skeleton shimmer moves between frames.
    #[test]
    fn skeleton_shimmer_moves_between_frames() {
        #[derive(Default)]
        struct Nothing;

        fn view(_: &Nothing) -> El<Nothing> {
            col(skeleton(Length::Fixed(100.0), 20.0))
        }

        let mut harness = Harness::new(Nothing, view).size(200.0, 100.0);
        harness.frame();

        // Get initial pixels
        let initial_pixels: Vec<u32> = harness.canvas().pixels().to_vec();

        // Step forward 6 frames
        harness.frames(6);

        // Get pixels after animation
        let animated_pixels: Vec<u32> = harness.canvas().pixels().to_vec();

        // At least one pixel should differ due to the shimmer animation
        let differs = initial_pixels
            .iter()
            .zip(animated_pixels.iter())
            .any(|(a, b)| a != b);

        assert!(differs, "skeleton shimmer should move across frames");
    }
}
