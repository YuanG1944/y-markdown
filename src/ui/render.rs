use super::theme;
use crate::ui::editor::elements::InputBridge;
use crate::{constants::EDITOR_PADDING, ui::editor::views::EditorView};

use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cursor_pos = self.document.cursor_position();
        let line_count = self.document.line_count();

        let mut col = div()
            .flex()
            .flex_col()
            .bg(theme::bg_color())
            .size_full()
            .relative()
            .p(px(EDITOR_PADDING))
            .text_xl()
            .text_color(theme::text_color())
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_keydown))
            .child(
                div()
                    .absolute()
                    .top(px(0.0))
                    .left(px(0.0))
                    .w(px(0.0))
                    .h(px(0.0))
                    .child(InputBridge {
                        entity: cx.entity(),
                        focus: self.focus_handle.clone(),
                    }),
            );

        for i in 0..line_count {
            col = col.child(self.render_line(i, cursor_pos, cx));
        }
        col
    }
}
