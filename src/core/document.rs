use std::ops::Range;

use crate::core::buffer::Buffer;
use crate::core::selection::Selection;
use ropey::RopeSlice;

pub struct Document {
    pub buffer: Buffer,
    pub selection: Selection,
}

impl Document {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            selection: Selection::new(0),
        }
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        self.selection.cursor_position(&self.buffer.text)
    }

    pub fn move_up(&mut self) {
        self.selection.move_up(&self.buffer.text);
    }

    pub fn move_down(&mut self) {
        self.selection.move_down(&self.buffer.text);
    }

    pub fn move_left(&mut self) {
        self.selection.move_left(1);
    }

    pub fn move_right(&mut self) {
        self.selection.move_right(1, self.buffer.len_chars());
    }

    pub fn insert(&mut self, text: &str) {
        self.buffer.insert_at(self.selection.end, text);
        self.selection.move_right(text.chars().count(), usize::MAX);
    }

    pub fn insert_newline(&mut self) {
        self.insert("\n");
    }

    pub fn delete_backward(&mut self) {
        if self.selection.end > 0 {
            let start = self.selection.end - 1;
            let end = self.selection.end;
            self.buffer.delete_range(start, end);
            self.selection.move_left(1);
        }
    }

    pub fn line_count(&self) -> usize {
        self.buffer.text.len_lines()
    }

    pub fn line(&'_ self, index: usize) -> RopeSlice<'_> {
        self.buffer.text.line(index)
    }

    pub fn split_at_cursor<'a>(&'a self) -> (RopeSlice<'a>, RopeSlice<'a>) {
        let cursor_pos = self.selection.end;
        (
            self.buffer.text.slice(0..cursor_pos),
            self.buffer.text.slice(cursor_pos..),
        )
    }

    // 1. 获取当前选区范围 (供 InputHandler 使用)
    pub fn selection_range(&self) -> Range<usize> {
        self.selection.start..self.selection.end
    }

    // 2. 替换指定范围内的文本 (供 InputHandler 使用)
    pub fn replace_range(&mut self, range: Range<usize>, text: &str) {
        // 先删除范围内旧的
        if range.end > range.start {
            self.buffer.delete_range(range.start, range.end);
        }

        // 再在 range.start 处插入新的
        self.buffer.insert_at(range.start, text);

        // 更新光标位置：移动到新插入文本的后面
        let new_cursor_pos = range.start + text.chars().count();
        self.selection = Selection::new(new_cursor_pos);
    }
}
