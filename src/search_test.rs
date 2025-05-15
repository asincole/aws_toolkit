use crate::search::SearchBar;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_bar_default() {
        let search_bar = SearchBar::default();
        assert_eq!(search_bar.query, "");
        assert_eq!(search_bar.active, false);
        assert_eq!(search_bar.cursor_position, 0);
    }

    #[test]
    fn test_search_bar_toggle() {
        let mut search_bar = SearchBar::default();
        search_bar.toggle();
        assert_eq!(search_bar.active, true);
        search_bar.toggle();
        assert_eq!(search_bar.active, false);
        assert_eq!(search_bar.query, "");
        assert_eq!(search_bar.cursor_position, 0);
    }

    #[test]
    fn test_search_bar_input() {
        let mut search_bar = SearchBar::default();
        search_bar.toggle();
        search_bar.input('a');
        assert_eq!(search_bar.query, "a");
        assert_eq!(search_bar.cursor_position, 1);
        search_bar.input('b');
        assert_eq!(search_bar.query, "ab");
        assert_eq!(search_bar.cursor_position, 2);
    }

    #[test]
    fn test_search_bar_delete() {
        let mut search_bar = SearchBar::default();
        search_bar.toggle();
        search_bar.input('a');
        search_bar.input('b');
        search_bar.delete();
        assert_eq!(search_bar.query, "a");
        assert_eq!(search_bar.cursor_position, 1);
        search_bar.delete();
        assert_eq!(search_bar.query, "");
        assert_eq!(search_bar.cursor_position, 0);
    }

    #[test]
    fn test_search_bar_clear() {
        let mut search_bar = SearchBar::default();
        search_bar.toggle();
        search_bar.input('a');
        search_bar.input('b');
        search_bar.clear();
        assert_eq!(search_bar.query, "");
        assert_eq!(search_bar.cursor_position, 0);
    }

    #[test]
    fn test_search_bar_matches() {
        let mut search_bar = SearchBar::default();
        search_bar.toggle();

        // Empty query matches everything
        assert_eq!(search_bar.matches("test"), true);

        // Exact match
        search_bar.input('t');
        search_bar.input('e');
        search_bar.input('s');
        search_bar.input('t');
        assert_eq!(search_bar.matches("test"), true);

        // Fuzzy match
        assert_eq!(search_bar.matches("testing"), true);

        // No match
        assert_eq!(search_bar.matches("foo"), false);
    }
}
