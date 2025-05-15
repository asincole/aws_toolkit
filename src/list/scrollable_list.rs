use crate::search::SearchBar;
use ratatui::widgets::ListState;

/// A scrollable list with filtering capabilities
#[derive(Debug)]
pub struct ScrollableList<T> {
    pub items: Vec<T>,
    pub filtered_indices: Vec<usize>,
    pub state: ListState,
    pub title: String,
    pub loading_more: bool,
    pub has_more: bool,
}

impl<T> Default for ScrollableList<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            filtered_indices: Vec::new(),
            state: ListState::default(),
            title: String::new(),
            loading_more: false,
            has_more: true,
        }
    }
}

impl<T> ScrollableList<T> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            items: Vec::new(),
            filtered_indices: Vec::new(),
            state: ListState::default(),
            title: title.into(),
            loading_more: false,
            has_more: true,
        }
    }

    pub fn with_items(mut self, items: Vec<T>) -> Self {
        self.items = items;
        self
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
    }

    pub fn next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                let current_pos = self.filtered_indices.iter().position(|&idx| idx == i);
                match current_pos {
                    Some(pos) => {
                        if pos >= self.filtered_indices.len() - 1 {
                            self.filtered_indices[0]
                        } else {
                            self.filtered_indices[pos + 1]
                        }
                    }
                    None => self.filtered_indices[0],
                }
            }
            None => self.filtered_indices[0],
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                let current_pos = self.filtered_indices.iter().position(|&idx| idx == i);
                match current_pos {
                    Some(pos) => {
                        if pos == 0 {
                            self.filtered_indices[self.filtered_indices.len() - 1]
                        } else {
                            self.filtered_indices[pos - 1]
                        }
                    }
                    None => self.filtered_indices[0],
                }
            }
            None => self.filtered_indices[0],
        };
        self.state.select(Some(i));
    }

    pub fn first(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.state.select(Some(self.filtered_indices[0]));
        }
    }

    pub fn last(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.state
                .select(Some(self.filtered_indices[self.filtered_indices.len() - 1]));
        }
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn append_items(&mut self, mut new_items: Vec<T>) {
        self.items.append(&mut new_items);
        self.loading_more = false;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading_more = loading;
    }

    pub fn apply_search<F>(&mut self, search_bar: &SearchBar, item_to_string: F)
    where
        F: Fn(&T) -> String,
    {
        self.filtered_indices.clear();

        for (idx, item) in self.items.iter().enumerate() {
            let item_str = item_to_string(item);
            if search_bar.matches(&item_str) {
                self.filtered_indices.push(idx);
            }
        }
        
        if let Some(selected) = self.state.selected() {
            if !self.filtered_indices.contains(&selected) {
                if !self.filtered_indices.is_empty() {
                    self.state.select(Some(self.filtered_indices[0]));
                } else {
                    self.state.select(None);
                }
            }
        }
    }

    pub fn set_has_more(&mut self, has_more: bool) {
        self.has_more = has_more;
    }
}
