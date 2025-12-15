// src/ui/editor/input.rs
use super::EditorView;
use crate::constants::LINE_HEIGHT;
use crate::utils::*;
use gpui::{Bounds, Context, EntityInputHandler, Pixels, Size, UTF16Selection, Window, px};
use std::ops::Range;

// Rust 允许在子模块中实现父模块定义的结构体的 Trait
impl EntityInputHandler for EditorView {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let text = self.document.buffer.text.to_string();
        let range = range_from_utf16(&text, &range_utf16);
        let s: String = text
            .chars()
            .skip(range.start)
            .take(range.end - range.start)
            .collect();
        actual_range.replace(range_to_utf16(&text, &range));
        Some(s)
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let text = self.document.buffer.text.to_string();

        if self.ime_active {
            let pos16 = char_index_to_utf16(&text, self.ime_caret);
            return Some(UTF16Selection {
                range: pos16..pos16,
                reversed: false,
            });
        }
        if let Some(marked) = &self.marked_range {
            let end16 = char_index_to_utf16(&text, marked.end);
            return Some(UTF16Selection {
                range: end16..end16,
                reversed: false,
            });
        }
        let sel = self.document.selection_range();
        Some(UTF16Selection {
            range: range_to_utf16(&text, &sel),
            reversed: false,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        let text = self.document.buffer.text.to_string();
        self.marked_range.as_ref().map(|r| range_to_utf16(&text, r))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        self.ime_active = false;
        let sel = self.document.selection_range();
        self.ime_caret = sel.end;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = self.document.buffer.text.to_string();
        let range = if let Some(r16) = range_utf16 {
            range_from_utf16(&text, &r16)
        } else if let Some(r) = self.marked_range.clone() {
            r
        } else {
            self.document.selection_range()
        };

        self.document.replace_range(range, new_text);
        self.marked_range = None;
        self.ime_active = false;
        let sel = self.document.selection_range();
        self.ime_caret = sel.end;
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_sel_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.ime_active = true;
        let text = self.document.buffer.text.to_string();

        let base = if let Some(r16) = range_utf16.clone() {
            range_from_utf16(&text, &r16)
        } else if let Some(r) = self.marked_range.clone() {
            r
        } else {
            self.document.selection_range()
        };

        self.document.replace_range(base.clone(), new_text);

        // 重新获取 text，因为内容变了 (虽然在这个简单的例子中直接计算也可以，但为了严谨)
        // 注意：实际项目中频繁 to_string 可能有性能问题，最好直接操作 Rope/Buffer
        let text_new = self.document.buffer.text.to_string();

        let start = base.start;
        let end = start + new_text.chars().count();
        self.marked_range = if new_text.is_empty() {
            None
        } else {
            Some(start..end)
        };

        let caret_char_idx = if let Some(sel16) = new_sel_utf16 {
            if range_utf16.is_none() {
                base.start + utf16_to_char_index(&text_new, sel16.end)
            } else {
                utf16_to_char_index(&text_new, sel16.end)
            }
        } else {
            end
        };
        self.ime_caret = caret_char_idx;
        self.document.set_cursor_position(self.ime_caret);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        if let Some(bounds) = self.last_cursor_bounds {
            let height = if bounds.size.height == px(0.0) {
                px(LINE_HEIGHT)
            } else {
                bounds.size.height
            };
            return Some(Bounds {
                origin: bounds.origin,
                size: Size {
                    width: px(1.0),
                    height,
                },
            });
        }
        None
    }

    fn character_index_for_point(
        &mut self,
        _pt: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.document.selection_range().start)
    }
}
