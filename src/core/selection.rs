// src/core/selection.rs
#[derive(Clone, Debug, Default)]
pub struct Selection {
    pub start: usize,
    pub end: usize,
}

impl Selection {
    pub fn new(cursor: usize) -> Self {
        Self {
            start: cursor,
            end: cursor,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn move_left(&mut self, amount: usize) {
        if self.end >= amount {
            self.end -= amount;
            self.start = self.end;
        }
    }

    pub fn move_right(&mut self, amount: usize, max_len: usize) {
        if self.end + amount <= max_len {
            self.end += amount;
            self.start = self.end;
        }
    }
}
