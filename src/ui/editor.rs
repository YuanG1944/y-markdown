use super::theme;
use crate::core::document::Document;
use gpui::{prelude::FluentBuilder, *};
use std::time::Instant;

pub struct EditorView {
    document: Document,
    pub focus_handle: FocusHandle,
    cursor_visible: bool,
    last_interaction: Instant,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let view = Self {
            document: Document::new(),
            focus_handle: cx.focus_handle(),
            cursor_visible: true,
            last_interaction: Instant::now(),
        };

        cx.spawn(|view: WeakEntity<EditorView>, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(500))
                        .await;

                    let update_result = cx.update(|cx| {
                        if let Some(view) = view.upgrade() {
                            view.update(cx, |editor, cx| {
                                if editor.last_interaction.elapsed().as_secs_f32() < 0.5 {
                                    editor.cursor_visible = true;
                                } else {
                                    editor.cursor_visible = !editor.cursor_visible;
                                }
                                cx.notify();
                            });
                        }
                    });

                    if update_result.is_err() {
                        break;
                    }
                }
            }
        })
        .detach();

        view
    }

    fn handle_keydown(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_interaction = Instant::now();
        self.cursor_visible = true;

        let key = event.keystroke.key.as_str();
        match key {
            "left" => self.document.move_left(),
            "right" => self.document.move_right(),
            "up" => self.document.move_up(),
            "down" => self.document.move_down(),
            "enter" => self.document.insert_newline(),
            "backspace" => self.document.delete_backward(),
            _ => {
                if let Some(c) = &event.keystroke.key_char {
                    self.document.insert(c);
                }
            }
        }
        cx.notify();
    }

    // --- 渲染辅助方法 ---

    // 1. 获取用于显示的文本（去除末尾换行符）
    fn get_display_text(&self, row: usize) -> String {
        let line_content = self.document.line(row);
        if line_content.len_chars() > 0 && line_content.to_string().ends_with('\n') {
            let s = line_content.to_string();
            s[..s.len() - 1].to_string()
        } else {
            line_content.to_string()
        }
    }

    fn cursor_div(&self, before_text: String) -> impl IntoElement {
        div()
            .absolute()
            .left(px(0.0))
            .top(px(0.0))
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            // 幽灵支架 (透明)
            .child(div().text_color(gpui::rgba(0x00000000)).child(before_text))
            // 光标本体
            .child(
                div().w(px(0.0)).flex_none().flex().items_center().child(
                    div()
                        .when(self.cursor_visible, |d| d.bg(theme::cursor_color()))
                        .absolute()
                        .top(px(-12.0))
                        .h(px(24.0))
                        .w(px(1.0)),
                ),
            )
    }

    // 2. 渲染光标行 (Layer 1 + Layer 2 Overlay)
    fn render_active_line(&self, text: String, cursor_col: usize) -> impl IntoElement {
        let safe_col = std::cmp::min(cursor_col, text.chars().count());
        let (before, _) = text.split_at(safe_col);
        let before_text = before.to_string();

        div()
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            // text layer
            .child(text)
            // cursor layer
            .child(self.cursor_div(before_text))
    }

    // 3. 渲染普通行
    fn render_static_line(&self, text: String) -> impl IntoElement {
        let text_to_show = if text.is_empty() { " " } else { &text };
        div().child(text_to_show.to_string())
    }

    // 4. 渲染单行容器 (统一的高度和对齐)
    fn render_line(&self, row: usize, cursor_pos: (usize, usize)) -> impl IntoElement {
        let (cursor_row, cursor_col) = cursor_pos;
        let is_active = row == cursor_row;
        let display_text = self.get_display_text(row);

        // 统一的外壳：确保所有行高度一致，防止抖动
        let row_container = div()
            .h(px(28.0))
            .flex()
            .flex_row()
            .items_center()
            .relative();

        if is_active {
            return row_container.child(self.render_active_line(display_text, cursor_col));
        }
        return row_container.child(self.render_static_line(display_text));
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cursor_pos = self.document.cursor_position();
        let line_count = self.document.line_count();

        let mut col_container = div()
            .flex()
            .flex_col()
            .bg(theme::bg_color())
            .size_full()
            .p(px(16.0))
            .text_xl()
            .text_color(theme::text_color())
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_keydown));

        // 循环渲染每一行，逻辑被委托给了 render_line
        for i in 0..line_count {
            col_container = col_container.child(self.render_line(i, cursor_pos));
        }

        col_container
    }
}
