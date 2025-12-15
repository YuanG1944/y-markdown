use super::EditorView;
use crate::constants::LINE_HEIGHT;
use crate::utils::*;
use gpui::{Bounds, Context, EntityInputHandler, Pixels, Size, UTF16Selection, Window, px};
use std::ops::Range;

impl EntityInputHandler for EditorView {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let buffer = &self.document.buffer;
        let range = buffer.range_from_utf16(&range_utf16);

        let s = buffer.slice_chars(range.start, range.end);

        actual_range.replace(buffer.range_to_utf16(&range));
        Some(s)
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let buffer = &self.document.buffer;

        if self.ime_active {
            let pos16 = buffer.char_to_utf16(self.ime_caret);
            return Some(UTF16Selection {
                range: pos16..pos16,
                reversed: false,
            });
        }
        if let Some(marked) = &self.marked_range {
            let end16 = buffer.char_to_utf16(marked.end);
            return Some(UTF16Selection {
                range: end16..end16,
                reversed: false,
            });
        }
        let sel = self.document.selection_range();
        Some(UTF16Selection {
            range: buffer.range_to_utf16(&sel),
            reversed: false,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        let buffer = &self.document.buffer;
        self.marked_range.as_ref().map(|r| buffer.range_to_utf16(r))
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
        let buffer = &self.document.buffer;
        let range = if let Some(r16) = range_utf16 {
            buffer.range_from_utf16(&r16)
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
        let buffer = &self.document.buffer;

        let base = if let Some(r16) = range_utf16.clone() {
            buffer.range_from_utf16(&r16)
        } else if let Some(r) = self.marked_range.clone() {
            r
        } else {
            self.document.selection_range()
        };

        self.document.replace_range(base.clone(), new_text);

        let buffer_new = &self.document.buffer;

        let start = base.start;
        let end = start + new_text.chars().count();
        self.marked_range = if new_text.is_empty() {
            None
        } else {
            Some(start..end)
        };

        let caret_char_idx = if let Some(sel16) = new_sel_utf16 {
            if range_utf16.is_none() {
                // 直接通过 buffer 转换 UTF-16 索引为字符索引
                base.start + buffer_new.utf16_to_char(sel16.end)
            } else {
                // 直接通过 buffer 转换 UTF-16 索引为字符索引
                buffer_new.utf16_to_char(sel16.end)
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
