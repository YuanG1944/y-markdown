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

    pub fn text(&self) -> &ropey::Rope {
        &self.buffer.text
    }
}
