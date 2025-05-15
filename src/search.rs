use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

#[derive(Debug)]
pub struct SearchBar {
    pub query: String,
    pub active: bool,
    pub cursor_position: usize,
}

impl Default for SearchBar {
    fn default() -> Self {
        Self {
            query: String::new(),
            active: false,
            cursor_position: 0,
        }
    }
}

impl SearchBar {
    pub fn toggle(&mut self) {
        self.active = !self.active;
        if !self.active {
            self.query.clear();
            self.cursor_position = 0;
        }
    }

    pub fn input(&mut self, c: char) {
        if self.active {
            self.query.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    pub fn delete(&mut self) {
        if self.active && self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.query.remove(self.cursor_position);
        }
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_position = 0;
    }

    pub fn matches(&self, item: &str) -> bool {
        if self.query.is_empty() {
            return true;
        }

        let matcher = SkimMatcherV2::default();
        matcher.fuzzy_match(item, &self.query).is_some()
    }
}
