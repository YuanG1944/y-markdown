// src/ui/editor.rs
use super::theme;
use crate::core::document::Document;
use gpui::*;

pub struct EditorView {
    document: Document,
    pub focus_handle: FocusHandle,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            document: Document::new(),
            focus_handle: cx.focus_handle(),
        }
    }

    fn handle_keydown(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = event.keystroke.key.as_str();

        match key {
            "left" => self.document.move_left(),
            "right" => self.document.move_right(),
            "backspace" => self.document.delete_backward(),
            _ => {
                if let Some(c) = &event.keystroke.key_char {
                    self.document.insert(c);
                }
            }
        }
        cx.notify();
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 1. 获取切分后的文本
        let (before, after) = self.document.split_at_cursor();

        // 2. 转换为 String (临时做法，为了渲染)
        // 注意：RopeSlice 转 String 会发生内存拷贝，对长文本性能不好
        // 但我们在做“多行渲染”优化前，这是最快恢复光标的办法
        let before_text = before.to_string();
        let after_text = after.to_string();

        div()
            .flex()
            .bg(theme::bg_color())
            .size_full()
            .justify_center()
            .items_center()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_keydown))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .child(
                        div()
                            .text_xl()
                            .text_color(theme::text_color())
                            .child(before_text),
                    )
                    .child(div().w(px(2.0)).h(px(24.0)).bg(theme::cursor_color()))
                    .child(
                        div()
                            .text_xl()
                            .text_color(theme::text_color())
                            .child(after_text),
                    ),
            )
    }
}
