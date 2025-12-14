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
}
