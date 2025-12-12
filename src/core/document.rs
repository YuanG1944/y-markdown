use ropey::RopeSlice;

// src/core/document.rs
use crate::core::buffer::Buffer;
use crate::core::history::EditHistory;
use crate::core::selection::Selection;

pub struct Document {
    pub buffer: Buffer,
    pub selection: Selection,
    // pub history: EditHistory,
}

impl Document {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            selection: Selection::new(0), // 初始光标在 0
                                          // history: EditHistory::new(),
        }
    }

    pub fn insert(&mut self, text: &str) {
        // 在当前光标位置插入文本
        self.buffer.insert_at(self.selection.end, text);

        // 移动光标
        self.selection.move_right(text.chars().count(), usize::MAX);
    }

    pub fn delete_backward(&mut self) {
        if self.selection.end > 0 {
            let start = self.selection.end - 1;
            let end = self.selection.end;

            // 从 buffer 删除
            self.buffer.delete_range(start, end);

            // update
            self.selection.move_left(1);
        }
    }

    pub fn move_left(&mut self) {
        self.selection.move_left(1);
    }

    pub fn move_right(&mut self) {
        self.selection.move_right(1, self.buffer.len_chars());
    }

    // 根据 Selection 的位置，返回两段文本切片
    pub fn split_at_cursor<'a>(&'a self) -> (RopeSlice<'a>, RopeSlice<'a>) {
        let cursor_pos = self.selection.end;
        (
            self.buffer.text.slice(0..cursor_pos),
            self.buffer.text.slice(cursor_pos..),
        )
    }

    // Get Total line
    pub fn line_count(&self) -> usize {
        self.buffer.text.len_lines()
    }

    // Get line text
    pub fn line<'a>(&'a self, index: usize) -> RopeSlice<'a> {
        self.buffer.text.line(index)
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        let cursor_char_idx = self.selection.end;
        let text = &self.buffer.text;

        // 接把全局字符索引转成行号
        let row = text.char_to_line(cursor_char_idx);

        // 该行起始字符在全局的位置
        let line_start_char = text.line_to_char(row);

        // 算出相对列号
        let col = cursor_char_idx - line_start_char;

        (row, col)
    }

    pub fn move_up(&mut self) {
        let (row, col) = self.cursor_position();

        // 只有不在第一行时才能向上移
        if row > 0 {
            self.move_vertical(row - 1, col);
        }
    }

    pub fn move_down(&mut self) {
        let (row, col) = self.cursor_position();
        let total_lines = self.line_count();

        // 只有不在最后一行时才能向下移
        if row < total_lines - 1 {
            self.move_vertical(row + 1, col);
        }
    }

    // 通用垂直移动逻辑
    fn move_vertical(&mut self, target_row: usize, target_col: usize) {
        // 1. 获取目标行的内容
        let line = self.buffer.text.line(target_row);
        let line_len = line.len_chars();

        // 2. 处理换行符：
        // ropey 的 line() 通常包含换行符。光标不应该停在换行符之后（那是下一行的开头）。
        // 所以有效长度通常是 line_len - 1 (如果有换行符的话)
        let has_newline = line.to_string().ends_with('\n'); // 简单判断
        let max_col = if has_newline && line_len > 0 {
            line_len - 1
        } else {
            line_len
        };

        // 3. 限制列号：不能超过目标行的长度
        // 比如从第10列移到只有5个字的行，光标应该停在第5个字
        let new_col = std::cmp::min(target_col, max_col);

        // 4. 将 (行, 列) 转换回全局索引
        let line_start_idx = self.buffer.text.line_to_char(target_row);
        let new_cursor_pos = line_start_idx + new_col;

        // 5. 更新 Selection
        self.selection = Selection::new(new_cursor_pos);
    }

    pub fn text(&self) -> &ropey::Rope {
        &self.buffer.text
    }
}
