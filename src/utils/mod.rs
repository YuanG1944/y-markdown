use crate::core::document::Document;
use std::ops::Range;

pub fn utf16_to_char_index(text: &str, utf16_offset: usize) -> usize {
    let mut chars = 0;
    let mut utf16 = 0;
    for c in text.chars() {
        if utf16 >= utf16_offset {
            break;
        }
        utf16 += c.len_utf16();
        chars += 1;
    }
    chars
}

/// 将字符索引转换为 UTF-16 偏移量
pub fn char_index_to_utf16(text: &str, idx: usize) -> usize {
    text.chars().take(idx).map(|c| c.len_utf16()).sum()
}

pub fn range_from_utf16(text: &str, r: &Range<usize>) -> Range<usize> {
    utf16_to_char_index(text, r.start)..utf16_to_char_index(text, r.end)
}

pub fn range_to_utf16(text: &str, r: &Range<usize>) -> Range<usize> {
    char_index_to_utf16(text, r.start)..char_index_to_utf16(text, r.end)
}

/// 计算字符偏移量对应的行列位置
pub fn offset_to_point(doc: &Document, char_offset: usize) -> (usize, usize) {
    let mut row: usize = 0;
    let mut offset = 0;
    for i in 0..doc.line_count() {
        let line = doc.line(i);
        let len = line.chars().count();
        if offset + len >= char_offset {
            let col = if char_offset >= offset {
                char_offset - offset
            } else {
                0
            };
            return (i, col);
        }
        offset += len;
        row += 1;
    }
    (row.saturating_sub(1), 0)
}

/// 获取用于显示的行文本（去除换行符）
pub fn get_display_text(doc: &Document, row: usize) -> String {
    let line = doc.line(row);
    let s = line.to_string();
    if s.ends_with('\n') {
        s[..s.len() - 1].to_string()
    } else {
        s
    }
}
