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
        // 获取光标位置信息
        let (cursor_row, cursor_col) = self.document.cursor_position();
        let line_count = self.document.line_count();

        // 创建一个垂直列表容器
        let mut col_container = div()
            .flex()
            .flex_col() // 垂直排列
            .bg(theme::bg_color())
            .size_full()
            .p(px(16.0)) // 增加一点内边距，不要贴着边
            .text_xl()
            .text_color(theme::text_color())
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_keydown));

        // 循环渲染每一行
        for i in 0..line_count {
            let line_content = self.document.line(i);

            // 去掉每行末尾的换行符，避免渲染出奇怪的符号
            // 但如果最后一行没有换行符，len_chars 为 0，要注意防崩
            let display_text =
                if line_content.len_chars() > 0 && line_content.to_string().ends_with('\n') {
                    let s = line_content.to_string();
                    s[..s.len() - 1].to_string() // 去掉最后一个 \n
                } else {
                    line_content.to_string()
                };

            // 4. 判断是否是光标行
            let row_element = if i == cursor_row {
                // == 当前是光标行：老规矩，切分 + 插入光标 ==
                // 注意：cursor_col 可能因为上面去掉了 \n 而越界，要做个防御
                let safe_col = std::cmp::min(cursor_col, display_text.chars().count());

                let (before, after) = display_text.split_at(safe_col);

                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(28.0))
                    .child(before.to_string())
                    .child(
                        // cursor
                        div().w(px(2.0)).h(px(24.0)).bg(theme::cursor_color()),
                    )
                    .child(after.to_string())
            } else {
                // == 普通行：直接显示文本 ==
                // 如果这一行为空（比如用户连按回车），我们需要渲染一个高度，否则这一行会塌陷
                let text_to_show = if display_text.is_empty() {
                    " "
                } else {
                    &display_text
                };

                div().h(px(28.0)).child(text_to_show.to_string())
            };

            col_container = col_container.child(row_element);
        }

        col_container
    }
}
