use crate::constants::{color::*, size::*};
use crate::core::document::Document;
use crate::ui::components::virtual_list::VirtualListScrollHandle;
use crate::ui::editor::elements::CursorTracker;
use crate::ui::theme;
use crate::utils::*;

use gpui::{
    Bounds, Context, FocusHandle, KeyDownEvent, Pixels, WeakEntity, Window, div,
    prelude::FluentBuilder, *,
};
use std::ops::Range;
use std::rc::Rc;
use std::time::Instant;

pub struct EditorView {
    pub(crate) document: Document,
    pub focus_handle: FocusHandle,
    pub(crate) cursor_visible: bool,
    pub(crate) last_interaction: Instant,

    pub(crate) marked_range: Option<Range<usize>>,
    pub(crate) ime_active: bool,
    pub(crate) ime_caret: usize,

    pub(crate) item_sizes: Rc<Vec<Size<Pixels>>>,
    pub(crate) current_line_height: Pixels,

    pub(crate) last_cursor_bounds: Option<Bounds<Pixels>>,
    pub(crate) list_scroll_handle: VirtualListScrollHandle,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // 1. 初始化结构体
        // 注意：这里改为 mut，因为我们需要在下面立即更新 layout
        let mut view = Self {
            document: Document::new(),
            focus_handle: cx.focus_handle(),
            cursor_visible: true,
            last_interaction: Instant::now(),
            marked_range: None,
            ime_active: false,
            ime_caret: 0,
            last_cursor_bounds: None,
            item_sizes: Rc::new(Vec::new()),
            current_line_height: px(0.0),
            list_scroll_handle: VirtualListScrollHandle::new(),
        };

        view.update_layout(cx);

        cx.spawn(|view: WeakEntity<EditorView>, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(CURSOR_BLINK_INTERVAL_MS))
                        .await;

                    let ok = cx
                        .update(|cx| {
                            if let Some(view) = view.upgrade() {
                                view.update(cx, |editor, cx| {
                                    if editor.last_interaction.elapsed().as_secs_f32()
                                        < CURSOR_BLINK_PAUSE_SEC
                                    {
                                        editor.cursor_visible = true;
                                    } else {
                                        editor.cursor_visible = !editor.cursor_visible;
                                    }
                                    // 3. 通知视图更新
                                    cx.notify();
                                });
                            }
                        })
                        .is_ok();

                    if !ok {
                        break;
                    }
                }
            }
        })
        .detach();

        view
    }

    pub fn handle_keydown(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_interaction = Instant::now();
        self.cursor_visible = true;

        match event.keystroke.key.as_str() {
            "left" => self.document.move_left(),
            "right" => self.document.move_right(),
            "up" => self.document.move_up(),
            "down" => self.document.move_down(),
            "enter" => self.document.insert_newline(),
            "backspace" => {
                if let Some(r) = self.marked_range.take() {
                    self.document.replace_range(r, "");
                } else {
                    self.document.delete_backward();
                }
                let sel = self.document.selection_range();
                self.ime_caret = sel.end;
            }
            _ => {}
        }

        self.update_layout(cx);

        let (cursor_line, _) = self.document.cursor_position();
        self.list_scroll_handle
            .scroll_to_item(cursor_line, ScrollStrategy::Center);

        let id = cx.entity().entity_id();
        cx.defer(move |cx| cx.notify(id));
    }

    fn cursor_div(&self, before_text: String, cx: &Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .left(px(0.0))
            .top(px(0.0))
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            .child(div().text_color(rgba(FONT_COLOR)).child(before_text))
            .child(
                div()
                    .w(px(0.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .child(CursorTracker {
                        entity: cx.entity().clone(),
                    })
                    .child(
                        div()
                            .when(self.cursor_visible, |d| d.bg(theme::cursor_color()))
                            .absolute()
                            .top(px(-EDITOR_LINE_HEIGHT / 2.0))
                            .h(px(EDITOR_LINE_HEIGHT))
                            .w(px(CURSOR_WIDTH)),
                    ),
            )
    }

    fn render_active_line(
        &self,
        text: String,
        cursor_col: usize,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let safe_col = cursor_col.min(text.chars().count());
        let before: String = text.chars().take(safe_col).collect();
        div()
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            .child(text)
            .child(self.cursor_div(before, cx))
    }

    fn update_layout(&mut self, _cx: &mut Context<Self>) {
        let line_count = self.document.line_count();

        let line_height = px(EDITOR_LINE_HEIGHT);

        if self.item_sizes.len() != line_count || self.current_line_height != line_height {
            self.current_line_height = line_height;
            self.item_sizes = std::rc::Rc::new(vec![
                gpui::Size {
                    width: px(0.0),
                    height: line_height
                };
                line_count
            ]);
        }
    }

    pub fn render_line(
        &self,
        row: usize,
        cursor_pos: (usize, usize),
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let (cursor_row, cursor_col) = cursor_pos;
        let txt = get_display_text(&self.document, row);

        let row_div = div()
            .h(self.current_line_height)
            .flex()
            .flex_row()
            .items_center()
            .relative();

        let (target_row, target_col) = if self.ime_active {
            offset_to_point(&self.document, self.ime_caret)
        } else {
            (cursor_row, cursor_col)
        };

        if row == target_row {
            row_div.child(self.render_active_line(txt, target_col, cx))
        } else {
            let txt = if txt.is_empty() { " " } else { &txt };
            row_div.child(div().child(txt.to_string()))
        }
    }
}
