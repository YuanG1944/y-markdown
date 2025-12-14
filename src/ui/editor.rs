use super::theme;
use crate::core::document::Document;
use gpui::{
    App, Bounds, Context, Element, ElementId, ElementInputHandler, GlobalElementId, LayoutId,
    Pixels, Style, Window, div, prelude::FluentBuilder, relative, *,
};
use std::ops::Range;
use std::time::Instant;

/// 主编辑器视图
pub struct EditorView {
    document: Document,
    pub focus_handle: FocusHandle,
    cursor_visible: bool,
    last_interaction: Instant,

    // ===== IME（输入法）状态 =====
    /// 当前合成（marked）区间，使用“字符索引”
    marked_range: Option<Range<usize>>,
    /// 是否处于预输入阶段
    ime_active: bool,
    /// IME 认为的插入点（字符索引）
    ime_caret: usize,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let view = Self {
            document: Document::new(),
            focus_handle: cx.focus_handle(),
            cursor_visible: true,
            last_interaction: Instant::now(),
            marked_range: None,
            ime_active: false,
            ime_caret: 0,
        };

        // 光标闪烁（保留原逻辑）
        cx.spawn(|view: WeakEntity<EditorView>, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(500))
                        .await;

                    let ok = cx
                        .update(|cx| {
                            if let Some(view) = view.upgrade() {
                                view.update(cx, |editor, cx| {
                                    if editor.last_interaction.elapsed().as_secs_f32() < 0.5 {
                                        editor.cursor_visible = true;
                                    } else {
                                        editor.cursor_visible = !editor.cursor_visible;
                                    }
                                    let id = cx.entity().entity_id();
                                    cx.defer(move |cx| cx.notify(id));
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

    // ===== 键盘按键（保留原逻辑） =====
    fn handle_keydown(
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
                // 同步 ime_caret
                let sel = self.document.selection_range();
                self.ime_caret = sel.end;
            }
            _ => {}
        }

        let id = cx.entity().entity_id();
        cx.defer(move |cx| cx.notify(id));
    }

    // ===== UTF-16 <-> 字符索引 =====
    fn full_text(&self) -> String {
        self.document.buffer.text.to_string()
    }

    fn utf16_to_char_index(&self, utf16_offset: usize) -> usize {
        let mut chars = 0;
        let mut utf16 = 0;
        for c in self.full_text().chars() {
            if utf16 >= utf16_offset {
                break;
            }
            utf16 += c.len_utf16();
            chars += 1;
        }
        chars
    }

    fn char_index_to_utf16(&self, idx: usize) -> usize {
        self.full_text()
            .chars()
            .take(idx)
            .map(|c| c.len_utf16())
            .sum()
    }

    fn range_from_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.utf16_to_char_index(r.start)..self.utf16_to_char_index(r.end)
    }

    fn range_to_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.char_index_to_utf16(r.start)..self.char_index_to_utf16(r.end)
    }

    // ===== 渲染辅助（保留你的原逻辑） =====
    fn get_display_text(&self, row: usize) -> String {
        let line = self.document.line(row);
        let s = line.to_string();
        if s.ends_with('\n') {
            s[..s.len() - 1].to_string()
        } else {
            s
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
            .child(div().text_color(rgba(0x00000000)).child(before_text))
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

    fn render_active_line(&self, text: String, cursor_col: usize) -> impl IntoElement {
        let safe_col = cursor_col.min(text.chars().count());
        let before: String = text.chars().take(safe_col).collect();
        div()
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            .child(text)
            .child(self.cursor_div(before))
    }

    fn render_static_line(&self, text: String) -> impl IntoElement {
        let txt = if text.is_empty() { " " } else { &text };
        div().child(txt.to_string())
    }

    fn render_line(&self, row: usize, cursor_pos: (usize, usize)) -> impl IntoElement {
        let (cursor_row, cursor_col) = cursor_pos;
        let active = row == cursor_row;
        let txt = self.get_display_text(row);
        let row_div = div()
            .h(px(28.0))
            .flex()
            .flex_row()
            .items_center()
            .relative();
        if active {
            row_div.child(self.render_active_line(txt, cursor_col))
        } else {
            row_div.child(self.render_static_line(txt))
        }
    }
}

/// 用于在 paint 阶段注册输入处理器的桥接元素
struct InputBridge {
    entity: Entity<EditorView>,
    focus: FocusHandle,
}
impl IntoElement for InputBridge {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}
impl Element for InputBridge {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _insp: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut s = Style::default();
        s.size.width = relative(1.).into();
        s.size.height = window.line_height().into();
        (window.request_layout(s, [], _cx), ())
    }
    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _insp: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _req: &mut Self::RequestLayoutState,
        _win: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _insp: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _req: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        win: &mut Window,
        cx: &mut App,
    ) {
        win.handle_input(
            &self.focus,
            ElementInputHandler::new(bounds, self.entity.clone()),
            cx,
        );
    }
}

/* =========================
   gpui 0.2.2 的 IME 接入点
========================= */
impl EntityInputHandler for EditorView {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let text = self.full_text();
        let range = self.range_from_utf16(&range_utf16);
        let s: String = text
            .chars()
            .skip(range.start)
            .take(range.end - range.start)
            .collect();
        actual_range.replace(self.range_to_utf16(&range));
        Some(s)
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        // ⚠️ 关键：预输入时一律报告 IME 维护的插入点（ime_caret）
        if self.ime_active {
            let pos16 = self.char_index_to_utf16(self.ime_caret);
            return Some(UTF16Selection {
                range: pos16..pos16,
                reversed: false,
            });
        }
        if let Some(marked) = &self.marked_range {
            let end16 = self.char_index_to_utf16(marked.end);
            return Some(UTF16Selection {
                range: end16..end16,
                reversed: false,
            });
        }
        let sel = self.document.selection_range();
        Some(UTF16Selection {
            range: self.range_to_utf16(&sel),
            reversed: false,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.as_ref().map(|r| self.range_to_utf16(r))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked_range = None;
        self.ime_active = false;
        // 同步 ime_caret 为当前 selection
        let sel = self.document.selection_range();
        self.ime_caret = sel.end;

        let id = cx.entity().entity_id();
        cx.defer(move |cx| cx.notify(id));
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = if let Some(r16) = range_utf16 {
            self.range_from_utf16(&r16)
        } else if let Some(r) = self.marked_range.clone() {
            r
        } else {
            self.document.selection_range()
        };

        self.document.replace_range(range, new_text);
        self.marked_range = None;
        self.ime_active = false;

        // 更新 ime_caret 到当前 selection 末尾
        let sel = self.document.selection_range();
        self.ime_caret = sel.end;

        let id = cx.entity().entity_id();
        cx.defer(move |cx| cx.notify(id));
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

        let base = if let Some(r16) = range_utf16.clone() {
            self.range_from_utf16(&r16)
        } else if let Some(r) = self.marked_range.clone() {
            r
        } else {
            self.document.selection_range()
        };

        self.document.replace_range(base.clone(), new_text);

        let start = base.start;
        let end = start + new_text.chars().count();
        self.marked_range = if new_text.is_empty() {
            None
        } else {
            Some(start..end)
        };

        // ✅ 修正：相对 offset 需要加上 base.start
        let caret_char_idx = if let Some(sel16) = new_sel_utf16 {
            if range_utf16.is_none() {
                base.start + self.utf16_to_char_index(sel16.end)
            } else {
                self.utf16_to_char_index(sel16.end)
            }
        } else {
            end
        };

        self.ime_caret = caret_char_idx;
        self.document.set_cursor_position(self.ime_caret);

        // ✅ 替代 invalidate：强制下一帧重绘
        let id = cx.entity().entity_id();
        cx.defer(move |cx| cx.notify(id));
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        // 优先使用我们维护的 ime_caret（预输入期间更可靠）
        let caret = if self.ime_active {
            self.ime_caret
        } else if let Some(marked) = &self.marked_range {
            marked.end
        } else {
            // 退路：IME 传进来的 end；再不行用 selection
            let from_ime = self.utf16_to_char_index(range_utf16.end);
            let sel = self.document.selection_range();
            if from_ime == 0 { sel.end } else { from_ime }
        };

        // 把“全局字符偏移”换算成 (row, col)
        let mut row = 0usize;
        let mut col = caret;
        let mut acc = 0usize;
        for i in 0..self.document.line_count() {
            let len = self.document.line(i).chars().count();
            if acc + len >= caret {
                row = i;
                col = caret - acc;
                break;
            }
            acc += len;
        }

        // 粗略像素坐标（如需精确可在绘制阶段缓存真实 Bounds）
        let line_height = 28.0;
        let char_width = 10.0;
        Some(Bounds {
            origin: Point {
                x: px(16.0 + col as f32 * char_width),
                y: px(16.0 + row as f32 * line_height + line_height),
            },
            size: Size {
                width: px(1.0),
                height: px(line_height),
            },
        })
    }

    fn character_index_for_point(
        &mut self,
        _pt: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.document.selection_range().start)
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cursor_pos = self.document.cursor_position();
        let line_count = self.document.line_count();

        let mut col = div()
            .flex()
            .flex_col()
            .bg(theme::bg_color())
            .size_full()
            .p(px(16.0))
            .text_xl()
            .text_color(theme::text_color())
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_keydown))
            // paint 时注册输入处理器
            .child(InputBridge {
                entity: cx.entity(),
                focus: self.focus_handle.clone(),
            });

        for i in 0..line_count {
            col = col.child(self.render_line(i, cursor_pos));
        }
        col
    }
}
