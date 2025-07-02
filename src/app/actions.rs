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

impl AppActions {
    pub fn from_key_event(key_event: KeyEvent, search_active: bool) -> Self {
        use crossterm::event::{KeyCode, KeyModifiers};

        match (key_event.code, key_event.modifiers) {
            // Common actions that work regardless of search mode
            (KeyCode::Enter, _) => Self::Enter,
            (KeyCode::Esc, _) => Self::GoBack,
            (KeyCode::Backspace, _) => {
                if search_active {
                    Self::SearchDelete
                } else {
                    Self::NoAction
                }
            }

            // Search activation
            (KeyCode::Char('/'), KeyModifiers::NONE) => Self::StartSearch,
            (KeyCode::Char('?'), KeyModifiers::SHIFT) => Self::StartFilterSearch,

            // If search is active, most characters become search input
            _ if search_active => match (key_event.code, key_event.modifiers) {
                (KeyCode::Char(c), KeyModifiers::NONE) => Self::SearchInput(c),
                (KeyCode::Char(c), KeyModifiers::SHIFT) if c.is_alphabetic() => {
                    Self::SearchInput(c.to_uppercase().next().unwrap())
                }
                (KeyCode::Char(c), KeyModifiers::SHIFT) => Self::SearchInput(c),
                _ => Self::NoAction,
            },

            // If search is not active, handle normal navigation and commands
            _ => match (key_event.code, key_event.modifiers) {
                // Navigation - Neovim style
                (KeyCode::Char('h'), KeyModifiers::NONE) | (KeyCode::Left, _) => Self::MoveLeft,
                (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, _) => Self::MoveDown,
                (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, _) => Self::MoveUp,
                (KeyCode::Char('l'), KeyModifiers::NONE) | (KeyCode::Right, _) => Self::MoveRight,

                // Top/Bottom navigation
                (KeyCode::Char('g'), KeyModifiers::NONE) => Self::MoveToTop, // First 'g' in 'gg'
                (KeyCode::Char('G'), KeyModifiers::SHIFT) => Self::MoveToBottom,
                (KeyCode::Home, _) | (KeyCode::Char('0'), KeyModifiers::NONE) => {
                    Self::MoveToLineStart
                }
                (KeyCode::End, _) | (KeyCode::Char('$'), KeyModifiers::SHIFT) => {
                    Self::MoveToLineEnd
                }

                // Page navigation
                (KeyCode::Char('f'), KeyModifiers::CONTROL) | (KeyCode::PageDown, _) => {
                    Self::PageDown
                }
                (KeyCode::Char('b'), KeyModifiers::CONTROL) | (KeyCode::PageUp, _) => Self::PageUp,
                (KeyCode::Char('d'), KeyModifiers::CONTROL) => Self::HalfPageDown,
                (KeyCode::Char('u'), KeyModifiers::CONTROL) => Self::HalfPageUp,

                // File operations
                (KeyCode::Char('s'), KeyModifiers::NONE) => Self::Download, // Save/download
                (KeyCode::Char('w'), KeyModifiers::NONE) => Self::Download, // Write (alternative)

                // General actions
                (KeyCode::Char('q'), KeyModifiers::NONE) => Self::Exit,
                (KeyCode::Char('r'), KeyModifiers::NONE) => Self::Refresh,
                (KeyCode::Char(' '), KeyModifiers::NONE) => Self::LoadMore,
                (KeyCode::Char('c'), KeyModifiers::NONE) => Self::ClearSearch,
                (KeyCode::Char('o'), KeyModifiers::NONE) => Self::Enter, // Open

                // Non-search character that we want to download with
                (KeyCode::Char('d'), KeyModifiers::NONE) => Self::Download,

                _ => Self::NoAction,
            },
        }
    }
}
