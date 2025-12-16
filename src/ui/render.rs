use super::theme;
use crate::constants::size::*;
use crate::ui::components::virtual_list::VirtualList;
use crate::ui::editor::elements::InputBridge;
use crate::ui::editor::views::EditorView;

use gpui::{
    Axis, Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let item_sizes = self.item_sizes.clone();

        div()
            .flex()
            .flex_col()
            .bg(theme::bg_color())
            .size_full()
            .relative()
            .p(px(EDITOR_PADDING))
            .text_size(px(EDITOR_FONT_SIZE))
            .line_height(px(EDITOR_LINE_HEIGHT))
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
            )
            .child(VirtualList::new(
                cx.entity().clone(),
                "editor-vlist",
                Axis::Vertical,
                self.list_scroll_handle.clone(),
                item_sizes,
                move |view, visible_range, _window, cx| {
                    let cursor_pos = view.document.cursor_position();
                    visible_range
                        .map(|i| view.render_line(i, cursor_pos, cx).into_any_element())
                        .collect()
                },
            ))
    }
}
