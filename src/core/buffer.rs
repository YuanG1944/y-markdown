// src/core/buffer.rs
use ropey::Rope;

pub struct Buffer {
    pub text: Rope,
    // 注意：cursor_offset 已经被移除，移交给了 Selection 管理
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            text: Rope::from("Hello, Editor!"),
        }
    }

    pub fn content_string(&self) -> String {
        self.text.to_string()
    }

    pub fn len_chars(&self) -> usize {
        self.text.len_chars()
    }

    // 泛用性更强的插入方法：指定位置 + 字符串
    pub fn insert_at(&mut self, index: usize, text: &str) {
        let len = self.text.len_chars();
        // 安全检查：防止索引越界导致 Panic
        let safe_index = std::cmp::min(index, len);
        self.text.insert(safe_index, text);
    }

    // 泛用性更强的删除方法：指定范围
    pub fn delete_range(&mut self, start: usize, end: usize) {
        let len = self.text.len_chars();
        if start < end && end <= len {
            self.text.remove(start..end);
        }
    }
}
