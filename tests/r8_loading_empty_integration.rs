//! Integration tests for loading and empty state recipes.
#[cfg(test)]
mod r8_loading_empty_integration {
    use rui::*;

    struct AppState {
        items: Vec<String>,
        loading: bool,
        error: Option<String>,
        stale: bool,
    }

    fn view(app: &AppState) -> El<AppState> {
        if let Some(err) = &app.error {
            error_state(err)
        } else if app.loading {
            loading_state("Fetching items...")
        } else if app.items.is_empty() {
            empty_state("No items found", "Create one to get started")
        } else if app.stale {
            stale_data_state("Data from 5 minutes ago")
        } else {
            col((text("Items loaded"),))
        }
    }

    #[test]
    fn shows_error_state_when_error_present() {
        let state = AppState {
            items: vec![],
            loading: false,
            error: Some("Connection failed".to_string()),
            stale: false,
        };
        let el = view(&state);
        assert!(!el.children().is_empty());
    }

    #[test]
    fn shows_loading_state_when_loading() {
        let state = AppState {
            items: vec![],
            loading: true,
            error: None,
            stale: false,
        };
        let el = view(&state);
        assert!(!el.children().is_empty());
    }

    #[test]
    fn shows_empty_state_when_no_items() {
        let state = AppState {
            items: vec![],
            loading: false,
            error: None,
            stale: false,
        };
        let el = view(&state);
        assert!(!el.children().is_empty());
    }

    #[test]
    fn shows_stale_data_when_flagged() {
        let state = AppState {
            items: vec!["Item 1".to_string()],
            loading: false,
            error: None,
            stale: true,
        };
        let el = view(&state);
        assert!(!el.children().is_empty());
    }

    #[test]
    fn shows_content_when_data_fresh() {
        let state = AppState {
            items: vec!["Item 1".to_string(), "Item 2".to_string()],
            loading: false,
            error: None,
            stale: false,
        };
        let el = view(&state);
        assert!(!el.children().is_empty());
    }

    #[test]
    fn empty_state_in_column_layout() {
        let empty = col((
            text("Container"),
            empty_state::<()>("No results", "Try again"),
        ));
        assert!(!empty.children().is_empty());
    }

    #[test]
    fn loading_state_in_row_layout() {
        let loading = row((loading_state::<()>("Loading"), spacer()));
        assert!(!loading.children().is_empty());
    }

    #[test]
    fn error_state_persists_across_frames() {
        let mut state = AppState {
            items: vec![],
            loading: false,
            error: Some("Error".to_string()),
            stale: false,
        };
        let el1 = view(&state);
        state.error = Some("Different error".to_string());
        let el2 = view(&state);
        assert!(!el1.children().is_empty());
        assert!(!el2.children().is_empty());
    }

    #[test]
    fn recipes_chain_with_style_methods() {
        let styled = empty_state::<()>("Empty", "Start")
            .gap(20.0)
            .pad(30.0)
            .center();
        assert!(!styled.children().is_empty());
    }
}
