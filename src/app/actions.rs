use crossterm::event::KeyEvent;

/// Handles all business logic actions for the application
#[derive(Debug)]
pub enum AppActions {
    // Navigation (Neovim style)
    MoveLeft,        // h, Left arrow
    MoveDown,        // j, Down arrow
    MoveUp,          // k, Up arrow
    MoveRight,       // l, Right arrow
    MoveToTop,       // gg
    MoveToBottom,    // G
    MoveToLineStart, // 0, Home
    MoveToLineEnd,   // $, End
    PageDown,        // Ctrl+f, PageDown
    PageUp,          // Ctrl+b, PageUp
    HalfPageDown,    // Ctrl+d
    HalfPageUp,      // Ctrl+u

    // Search actions
    StartSearch,       // /
    StartFilterSearch, // ? (for local filtering)
    SearchInput(char),
    SearchDelete,
    // SearchSubmit,
    Enter,
    // SearchCancel,
    ClearSearch, // :noh equivalent

    // General actions
    Exit, // q
    // SelectItem, // Enter
    GoBack,   // Esc
    LoadMore, // Space
    Download, // d
    Refresh,  // r

    // Special
    NoAction,
}

impl From<KeyEvent> for AppActions {
    fn from(key_event: KeyEvent) -> Self {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key_event.code, key_event.modifiers) {
            // Navigation - Neovim style
            (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, _) => Self::MoveLeft,
            (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => Self::MoveDown,
            (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => Self::MoveUp,
            (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, _) => Self::MoveRight,

            // Top/Bottom navigation
            (KeyCode::Char('g'), KeyModifiers::NONE) => Self::MoveToTop, // First 'g' in 'gg'
            (KeyCode::Char('G'), KeyModifiers::SHIFT) => Self::MoveToBottom,
            (KeyCode::Home, _) | (KeyCode::Char('0'), KeyModifiers::NONE) => Self::MoveToLineStart,
            (KeyCode::End, _) | (KeyCode::Char('$'), KeyModifiers::SHIFT) => Self::MoveToLineEnd,

            // Page navigation
            (KeyCode::Char('f'), KeyModifiers::CONTROL) | (KeyCode::PageDown, _) => Self::PageDown,
            (KeyCode::Char('b'), KeyModifiers::CONTROL) | (KeyCode::PageUp, _) => Self::PageUp,
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => Self::HalfPageDown,
            (KeyCode::Char('u'), KeyModifiers::CONTROL) => Self::HalfPageUp,

            // Search
            (KeyCode::Char('/'), KeyModifiers::NONE) => Self::StartSearch, // Server-side search
            (KeyCode::Char('?'), KeyModifiers::SHIFT) => Self::StartFilterSearch, // Local filter
            // (KeyCode::Char('n'), KeyModifiers::NONE) => Self::SearchNext,         // Next search result
            // (KeyCode::Char('N'), KeyModifiers::SHIFT) => Self::SearchPrevious,    // Previous search result
            (KeyCode::Backspace, _) => Self::SearchDelete,
            (KeyCode::Enter, _) => Self::Enter,
            (KeyCode::Esc, _) => Self::GoBack,

            // File operations
            (KeyCode::Char('s'), KeyModifiers::NONE) => Self::Download, // Save/download
            (KeyCode::Char('w'), KeyModifiers::NONE) => Self::Download, // Write (alternative)
            // (KeyCode::Char('y'), KeyModifiers::NONE) => Self::CopyPath,           // Yank path to clipboard
            // (KeyCode::Char('p'), KeyModifiers::NONE) => Self::Preview,            // Preview toggle

            // General actions
            (KeyCode::Char('q'), KeyModifiers::NONE) => Self::Exit,
            // (KeyCode::Char('Q'), KeyModifiers::SHIFT) => Self::ForceExit,         // Force quit
            (KeyCode::Char('r'), KeyModifiers::NONE) => Self::Refresh,
            // (KeyCode::Char('R'), KeyModifiers::SHIFT) => Self::RefreshAll,        // Hard refresh
            (KeyCode::Char(' '), KeyModifiers::NONE) => Self::LoadMore,
            (KeyCode::Char('c'), KeyModifiers::NONE) => Self::ClearSearch,
            // (KeyCode::Char('i'), KeyModifiers::NONE) => Self::ShowInfo,           // Show object info
            (KeyCode::Char('o'), KeyModifiers::NONE) => Self::Enter, // Open
            // (KeyCode::Enter, _) => Self::SelectItem,

            // Tab navigation (if you have multiple views)
            // (KeyCode::Tab, _) => Self::NextTab,
            // (KeyCode::BackTab, _) => Self::PreviousTab,

            // Help
            // (KeyCode::Char('?'), KeyModifiers::NONE) => Self::ShowHelp,           // Show help (alternative to Shift+?)
            // (KeyCode::F(1), _) => Self::ShowHelp,

            // Sorting (useful for file browsers)
            // (KeyCode::Char('S'), KeyModifiers::SHIFT) => Self::ToggleSort,        // Toggle sort mode

            // Character input (for search)
            (KeyCode::Char(c), KeyModifiers::NONE) => Self::SearchInput(c),
            (KeyCode::Char(c), KeyModifiers::SHIFT) if c.is_alphabetic() => {
                Self::SearchInput(c.to_uppercase().next().unwrap())
            }

            _ => Self::NoAction,
        }
    }
}
