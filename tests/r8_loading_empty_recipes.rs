//! Tests for loading and empty state recipes (R10).
#[cfg(test)]
mod r8_loading_empty_recipes {
    use rui::*;

    #[test]
    fn empty_state_renders() {
        let _empty = empty_state::<()>("No items", "Add one");
    }

    #[test]
    fn empty_state_has_structure() {
        let empty = empty_state::<()>("Empty", "Create");
        assert!(!empty.children().is_empty());
    }

    #[test]
    fn empty_state_renders_different_messages() {
        let _empty1 = empty_state::<()>("No data", "Get started");
        let _empty2 = empty_state::<()>("No results", "Try again");
    }

    #[test]
    fn loading_state_renders() {
        let _loading = loading_state::<()>("Loading data...");
    }

    #[test]
    fn loading_state_has_structure() {
        let loading = loading_state::<()>("Please wait");
        assert!(!loading.children().is_empty());
    }

    #[test]
    fn stale_data_state_renders() {
        let stale = stale_data_state::<()>("Data from 2m ago");
        assert!(!stale.children().is_empty());
    }

    #[test]
    fn error_state_renders() {
        let error = error_state::<()>("Connection failed");
        assert!(!error.children().is_empty());
    }

    #[test]
    fn error_state_renders_different_messages() {
        let _error1 = error_state::<()>("Something went wrong");
        let _error2 = error_state::<()>("Failed to load");
    }
}
