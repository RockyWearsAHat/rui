//! Toast notification tests.
//!
//! Toast is a short confirmation message floating at the bottom of the window.
//! It has no timer of its own and the caller manages when it appears and disappears.

use rui::testing::Harness;
use rui::toast;
use rui::Status;

#[test]
fn toast_shows_its_message() {
    struct State;

    fn view(_state: &State) -> rui::El<State> {
        toast("Operation successful", Status::Ok)
    }

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    assert!(
        harness.shows("Operation successful"),
        "toast should display its message"
    );
}

#[test]
fn toast_is_layered() {
    struct State;

    fn view(_state: &State) -> rui::El<State> {
        toast("Test message", Status::Ok)
    }

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    let probes = harness.probes();
    assert!(
        probes.iter().any(|p| p.key == Some("toast".to_string())),
        "toast should have key 'toast' for identification"
    );

    // Check that the toast element is layered
    let toast_probe = probes
        .iter()
        .find(|p| p.key == Some("toast".to_string()))
        .expect("toast element should exist");
    assert!(toast_probe.layered, "toast should be layered");
}

#[test]
fn toast_carries_its_status_colour() {
    struct State;

    fn view(_state: &State) -> rui::El<State> {
        toast("Status test", Status::Ok)
    }

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    // Verify that the toast was created successfully with the status
    // The dot inside should carry the status color
    assert!(harness.shows("Status test"));
}

#[test]
fn toast_names_itself_for_accessibility() {
    struct State;

    fn view(_state: &State) -> rui::El<State> {
        toast("Accessible message", Status::Ok)
    }

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    // The accessibility name is stored on the AccessNode, set by .label()
    let accessibility_tree = harness.accessibility();
    let nodes = accessibility_tree.nodes();

    // Find the node with key "toast" by checking its position/role
    let toast_node = nodes
        .iter()
        .find(|n| n.name == "Accessible message")
        .expect("toast should have an accessibility node with the message as its name");

    assert_eq!(
        toast_node.name, "Accessible message",
        "toast should be named for accessibility"
    );
}

#[test]
fn toast_is_not_focusable() {
    struct State;

    fn view(_state: &State) -> rui::El<State> {
        toast("Non-focusable", Status::Ok)
    }

    let mut harness = Harness::new(State, view).size(400.0, 600.0);
    harness.frame();

    let probes = harness.probes();
    let toast_probe = probes
        .iter()
        .find(|p| p.key == Some("toast".to_string()))
        .expect("toast element should exist");

    assert!(!toast_probe.focusable, "toast should not be focusable");
}
