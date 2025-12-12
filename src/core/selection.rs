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

    // 是否只是一个光标（没有选中文字）
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    // 简单的光标移动逻辑（暂不处理选区扩展）
    pub fn move_left(&mut self, amount: usize) {
        if self.end >= amount {
            self.end -= amount;
            self.start = self.end; // 保持光标合并
        }
    }

    pub fn move_right(&mut self, amount: usize, max_len: usize) {
        if self.end + amount <= max_len {
            self.end += amount;
            self.start = self.end;
        }
    }
}
