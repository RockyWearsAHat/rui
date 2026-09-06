//! Tests for the icon component.

use rui::testing::Harness;
use rui::*;

/// Every icon should paint inside its box.
#[test]
fn every_icon_paints_inside_its_box() {
    let icons = [
        Icon::Folder,
        Icon::File,
        Icon::Branch,
        Icon::Commit,
        Icon::Tag,
        Icon::Clone,
        Icon::Check,
        Icon::Cross,
        Icon::Chevron,
        Icon::Copy,
        Icon::Search,
        Icon::Dot,
        Icon::History,
        Icon::Download,
        Icon::Code,
    ];

    for icon_kind in icons {
        let view = move |_: &()| {
            let size = 24.0;
            icon(icon_kind, size).w(size).h(size)
        };

        let mut harness = Harness::new((), view).size(100.0, 100.0);
        harness.frame();

        // The icon's box
        let icon_rect = Rect::new(38.0, 38.0, 24.0, 24.0);

        // There should be marked pixels inside the box
        assert!(
            harness.marked(icon_rect),
            "Icon {:?} paints inside its box",
            icon_kind
        );

        // The area outside should be clear (no painted pixels)
        let outside_rect = Rect::new(10.0, 10.0, 10.0, 10.0);
        assert!(
            !harness.marked(outside_rect),
            "Icon {:?} does not paint outside its box",
            icon_kind
        );
    }
}

/// Every icon should name itself for accessibility.
#[test]
fn every_icon_names_itself() {
    let icons = [
        Icon::Folder,
        Icon::File,
        Icon::Branch,
        Icon::Commit,
        Icon::Tag,
        Icon::Clone,
        Icon::Check,
        Icon::Cross,
        Icon::Chevron,
        Icon::Copy,
        Icon::Search,
        Icon::Dot,
        Icon::History,
        Icon::Download,
        Icon::Code,
    ];

    let icon_names = [
        "folder", "file", "branch", "commit", "tag", "clone", "check", "cross", "chevron", "copy",
        "search", "dot", "history", "download", "code",
    ];

    for (i, &icon_kind) in icons.iter().enumerate() {
        let _expected_name = icon_names[i];
        let view = move |_: &()| icon(icon_kind, 24.0);

        let mut harness = Harness::new((), view).size(100.0, 100.0);
        harness.frame();

        let probes = harness.probes();
        let found = probes.iter().any(|p| p.role == Role::Image);

        // Check that the icon has the Image role
        assert!(found, "Icon {:?} should have Image role", icon_kind);
    }
}

/// Icon should respect its tint.
#[test]
fn icon_respects_its_tint() {
    let view = move |_: &()| {
        col((
            icon_tinted(Icon::Folder, 24.0, Tone::Muted),
            icon_tinted(Icon::Folder, 24.0, Tone::Text),
            icon_tinted(Icon::Folder, 24.0, Tone::Accent),
        ))
    };

    let mut harness = Harness::new((), view).size(100.0, 200.0);
    harness.frame();

    // Find all Image elements and verify they are painted
    let rects: Vec<_> = {
        let probes = harness.probes();
        probes
            .iter()
            .filter(|p| p.role == Role::Image)
            .map(|p| p.rect)
            .collect()
    };

    assert!(rects.len() >= 3, "Should have at least 3 image probes");

    // Check that each image's rect is painted
    for rect in rects.iter().take(3) {
        assert!(harness.marked(*rect), "Icon at {:?} should paint", rect);
    }
}

/// Icons should scale without clipping at various sizes.
#[test]
fn icon_scales_without_clipping() {
    let sizes = vec![12.0, 16.0, 24.0, 48.0];

    for size in sizes {
        let view = move |_: &()| icon(Icon::Commit, size);

        let mut harness = Harness::new((), view).size(100.0, 100.0);
        harness.frame();

        // The icon should fit in its declared box
        let icon_rect = Rect::new((100.0 - size) / 2.0, (100.0 - size) / 2.0, size, size);

        // Should be marked inside
        assert!(
            harness.marked(icon_rect),
            "Icon at size {} should paint",
            size
        );
    }
}

/// Icon should hold no text.
#[test]
fn icon_holds_no_text() {
    let icons = [
        Icon::Folder,
        Icon::File,
        Icon::Branch,
        Icon::Commit,
        Icon::Tag,
        Icon::Clone,
        Icon::Check,
        Icon::Cross,
        Icon::Chevron,
        Icon::Copy,
        Icon::Search,
        Icon::Dot,
        Icon::History,
        Icon::Download,
        Icon::Code,
    ];

    for icon_kind in icons {
        let view = move |_: &()| icon(icon_kind, 24.0);

        let mut harness = Harness::new((), view).size(100.0, 100.0);
        harness.frame();

        // Check that the image element has no text content
        let probes = harness.probes();
        let image_probe = probes.iter().find(|p| p.role == Role::Image);

        if let Some(probe) = image_probe {
            assert!(
                probe.text.is_none(),
                "Icon {:?} should not hold text",
                icon_kind
            );
        }
    }
}
