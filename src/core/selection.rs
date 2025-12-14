use ropey::Rope;
use std::cmp::{max, min};

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

    // === 水平移动 ===

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

    // === 垂直移动 (逻辑搬迁至此) ===

    // 计算 (行, 列)
    pub fn cursor_position(&self, text: &Rope) -> (usize, usize) {
        let cursor_char_idx = self.end;

        // 防止越界
        if cursor_char_idx > text.len_chars() {
            return (0, 0);
        }

        let row = text.char_to_line(cursor_char_idx);
        let line_start_char = text.line_to_char(row);
        let col = cursor_char_idx - line_start_char;

        (row, col)
    }

    pub fn move_up(&mut self, text: &Rope) {
        let (row, col) = self.cursor_position(text);
        if row > 0 {
            self.move_vertical(row - 1, col, text);
        }
    }

    pub fn move_down(&mut self, text: &Rope) {
        let (row, col) = self.cursor_position(text);
        let total_lines = text.len_lines();

        if row < total_lines - 1 {
            self.move_vertical(row + 1, col, text);
        }
    }

    // 私有辅助逻辑
    fn move_vertical(&mut self, target_row: usize, target_col: usize, text: &Rope) {
        let line = text.line(target_row);
        let line_len = line.len_chars();

        // 处理换行符逻辑
        let has_newline = line.to_string().ends_with('\n');
        let max_col = if has_newline && line_len > 0 {
            line_len - 1
        } else {
            line_len
        };

        let new_col = min(target_col, max_col);
        let line_start_idx = text.line_to_char(target_row);
        let new_cursor_pos = line_start_idx + new_col;

        // 更新自身状态
        self.end = new_cursor_pos;
        self.start = new_cursor_pos; // 移动时取消选中
    }
}
