// src/core/buffer.rs
use ropey::Rope;

pub struct Buffer {
    pub text: Rope,
    // 注意：这里没有 selection
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            text: Rope::from("Hello, Type something..."),
        }
    }

    pub fn content_string(&self) -> String {
        self.text.to_string()
    }

    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    // 基础插入
    pub fn insert_at(&mut self, index: usize, text: &str) {
        let len = self.text.len_chars();
        let safe_index = std::cmp::min(index, len);
        self.text.insert(safe_index, text);
    }

    // 基础删除
    pub fn delete_range(&mut self, start: usize, end: usize) {
        let len = self.text.len_chars();
        if start < end && end <= len {
            self.text.remove(start..end);
        }
    }

    // 获取指定字符范围的文本片段
    pub fn slice_chars(&self, start: usize, end: usize) -> String {
        self.text.slice(start..end).to_string()
    }

    // 字符索 -> UTF-16 索引
    pub fn char_to_utf16(&self, char_idx: usize) -> usize {
        self.text.char_to_utf16_cu(char_idx)
    }

    // UTF-16 索引 -> 为字符索引
    pub fn utf16_to_char(&self, utf16_idx: usize) -> usize {
        self.text.utf16_cu_to_char(utf16_idx)
    }

    // TF-16 范围->为字符范围
    pub fn range_from_utf16(&self, range_utf16: &std::ops::Range<usize>) -> std::ops::Range<usize> {
        self.utf16_to_char(range_utf16.start)..self.utf16_to_char(range_utf16.end)
    }

    // 字符范围-> UTF-16 范围
    pub fn range_to_utf16(&self, range_char: &std::ops::Range<usize>) -> std::ops::Range<usize> {
        self.char_to_utf16(range_char.start)..self.char_to_utf16(range_char.end)
    }
}
