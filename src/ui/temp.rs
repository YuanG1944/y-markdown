// src/ui/editor.rs

use gpui::{prelude::FluentBuilder, *};
use ropey::Rope;
use std::{ops::Range, time::Instant};

// 假设你的 theme 模块在 ../theme.rs，如果路径不同请调整
use super::theme;

pub struct EditorView {
    // [核心改变] 使用 Rope 替代 Document，直接管理文本
    text: Rope,
    // [核心改变] 简单的光标偏移量 (Char Index)
    cursor_offset: usize,

    pub focus_handle: FocusHandle,
    cursor_visible: bool,
    last_interaction: Instant,

    // IME 相关字段 (保留你的逻辑)
    last_cursor_point: Point<Pixels>,
    marked_range: Option<Range<usize>>,
}

impl EditorView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // 初始化一些示例文本
        let initial_text = Rope::from_str("Hello Rust!");
        let len = initial_text.len_chars();

        let view = Self {
            text: initial_text,
            cursor_offset: len, // 默认光标在最后
            focus_handle: cx.focus_handle(),
            cursor_visible: true,
            last_interaction: Instant::now(),
            last_cursor_point: Point::default(),
            marked_range: None,
        };

        // 光标闪烁计时器 (保留你的逻辑)
        cx.spawn(|view: WeakEntity<EditorView>, cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                loop {
                    cx.background_executor()
                        .timer(std::time::Duration::from_millis(500))
                        .await;

                    if let Some(result) = cx
                        .update(|cx| {
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
                        })
                        .ok()
                    {
                        // update 成功则继续，否则退出
                    } else {
                        break;
                    }
                }
            }
        })
        .detach();

        view
    }

    // --- 核心编辑逻辑 (原 Document 的功能) ---

    // 获取光标的 (行, 列) 坐标
    fn cursor_position(&self) -> (usize, usize) {
        let row = self.text.char_to_line(self.cursor_offset);
        let line_start_char = self.text.line_to_char(row);
        let col = self.cursor_offset - line_start_char;
        (row, col)
    }

    fn move_left(&mut self) {
        if self.cursor_offset > 0 {
            self.cursor_offset -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cursor_offset < self.text.len_chars() {
            self.cursor_offset += 1;
        }
    }

    fn move_up(&mut self) {
        let (row, col) = self.cursor_position();
        if row > 0 {
            let prev_row = row - 1;
            let prev_line_len = self.text.line(prev_row).len_chars();
            // 简单的列记忆逻辑：如果上一行比较短，就跳到行尾；注意要减去换行符
            let new_col = std::cmp::min(col, prev_line_len.saturating_sub(1));
            let new_line_start = self.text.line_to_char(prev_row);
            self.cursor_offset = new_line_start + new_col;
        }
    }

    fn move_down(&mut self) {
        let (row, col) = self.cursor_position();
        if row < self.text.len_lines() - 1 {
            let next_row = row + 1;
            let next_line_len = self.text.line(next_row).len_chars();
            let new_col = std::cmp::min(col, next_line_len.saturating_sub(1));
            let next_line_start = self.text.line_to_char(next_row);
            self.cursor_offset = next_line_start + new_col;
        }
    }

    fn insert_newline(&mut self) {
        self.text.insert_char(self.cursor_offset, '\n');
        self.cursor_offset += 1;
    }

    fn delete_backward(&mut self) {
        if self.cursor_offset > 0 {
            self.text
                .remove((self.cursor_offset - 1)..self.cursor_offset);
            self.cursor_offset -= 1;
        }
    }

    fn insert(&mut self, text: &str) {
        self.text.insert(self.cursor_offset, text);
        self.cursor_offset += text.chars().count();
    }

    // --- 事件处理 ---

    fn handle_keydown(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // [新增调试] 看看按键到底进没进来
        println!("按键按下: {:?}", event.keystroke.key);

        self.last_interaction = Instant::now();
        self.cursor_visible = true;

        let key = event.keystroke.key.as_str();
        match key {
            "left" => self.move_left(),
            "right" => self.move_right(),
            "up" => self.move_up(),
            "down" => self.move_down(),
            "enter" => self.insert_newline(),
            "backspace" => self.delete_backward(),
            _ => {
                if let Some(c) = &event.keystroke.key_char {
                    if !event.keystroke.modifiers.control
                        && !event.keystroke.modifiers.alt
                        && !event.keystroke.modifiers.platform
                    {
                        self.insert(&c.to_string());
                    }
                }
            }
        }
        cx.notify();
    }

    // --- 渲染逻辑 (适配 Rope) ---

    fn get_display_text(&self, row: usize) -> String {
        // Ropey 的 line() 返回的是 Slice，包含换行符
        let line = self.text.line(row);
        let mut s = line.to_string();
        if s.ends_with('\n') {
            s.pop();
        }
        s
    }

    // ... 这里保留你原本的 cursor_div, render_active_line, render_static_line ...
    // 为了节省篇幅，假设这些辅助函数代码与你原来的一模一样
    // 只要把 render_line 稍微改一下

    fn cursor_div(&self, before_text: String) -> impl IntoElement {
        // (你的原始代码，保持不变)
        div()
            .absolute()
            .left(px(0.0))
            .top(px(0.0))
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            .child(div().text_color(gpui::rgba(0x00000000)).child(before_text))
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
        // 1. 确保 cursor_col 不超过当前行的字符总数
        let char_count = text.chars().count(); // 注意：这里需要遍历一次计算字符数
        let safe_col = std::cmp::min(cursor_col, char_count);

        // 2. [修正] 获取光标前的文本
        // 之前错误的写法: let (before, _) = text.split_at(safe_col);
        // 修正后的写法: 使用 chars迭代器 安全地取前 N 个字符
        let before_text: String = text.chars().take(safe_col).collect();

        div()
            .size_full()
            .flex()
            .flex_row()
            .items_center()
            // 渲染整行文本
            .child(text)
            // 渲染覆盖在上面的光标层
            .child(self.cursor_div(before_text))
    }

    fn render_static_line(&self, text: String) -> impl IntoElement {
        // (你的原始代码，保持不变)
        let text_to_show = if text.is_empty() { " " } else { &text };
        div().child(text_to_show.to_string())
    }

    fn render_line(&self, row: usize, cursor_pos: (usize, usize)) -> impl IntoElement {
        let (cursor_row, cursor_col) = cursor_pos;
        let is_active = row == cursor_row;
        let display_text = self.get_display_text(row);

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
        let cursor_pos = self.cursor_position();
        let line_count = self.text.len_lines();

        let mut col_container = div()
            .flex()
            .flex_col()
            .bg(theme::bg_color())
            .size_full()
            .p(px(16.0))
            .text_xl()
            .text_color(theme::text_color())
            // --- [关键修改] 指定字体 ---
            // Windows 必须指定 "Microsoft YaHei" (微软雅黑) 才能正常显示中文
            // macOS 可以用 "PingFang SC"
            .font_family("Microsoft YaHei, PingFang SC")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_keydown))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|view, _event, window, _cx| {
                    window.focus(&view.focus_handle);
                }),
            );

        for i in 0..line_count {
            col_container = col_container.child(self.render_line(i, cursor_pos));
        }

        col_container
    }
}

// --- InputHandler 实现 (适配 Rope) ---
impl InputHandler for EditorView {
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _app: &mut App,
    ) -> Option<UTF16Selection> {
        // 暂时只支持光标位置，没有选区
        let range = self.cursor_offset..self.cursor_offset;
        Some(UTF16Selection {
            range,
            reversed: false,
        })
    }

    fn marked_text_range(&mut self, _window: &mut Window, _app: &mut App) -> Option<Range<usize>> {
        self.marked_range.clone()
    }

    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _app: &mut App,
    ) -> Option<String> {
        let len = self.text.len_chars();
        let start = range_utf16.start.min(len);
        let end = range_utf16.end.min(len);

        println!("IME 请求文本: {:?} (Doc Len: {})", start..end, len); // [调试日志]

        // 之前的写法: if start < end { ... } else { None }
        // 这种写法会导致 0..0 返回 None，导致 IME 认为初始化失败

        // 修正后的写法：
        if start <= end {
            Some(self.text.slice(start..end).to_string())
        } else {
            None
        }
    }

    fn replace_text_in_range(
        &mut self,
        replacement_range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        _app: &mut App,
    ) {
        // [调试] 打印看看系统有没有把中文传进来
        println!("输入法提交文本: '{}'", text);

        // 逻辑优先级：
        // 1. 输入法明确指定要替换哪里 (replacement_range)
        // 2. 如果没指定，就替换当前正在预输入(下划线)的部分 (marked_range)
        // 3. 如果都没有，就在光标处插入 (selection)
        let range = replacement_range
            .or(self.marked_range.clone())
            .unwrap_or(self.cursor_offset..self.cursor_offset);

        // [Ropey 操作]
        let start = range.start.min(self.text.len_chars());
        let end = range.end.min(self.text.len_chars());

        if start < end {
            self.text.remove(start..end);
        }
        self.text.insert(start, text);

        // 移动光标到新文本后面
        self.cursor_offset = start + text.chars().count();

        // [关键] 提交完成后，清除标记状态
        self.marked_range = None;

        self.last_interaction = Instant::now();
        self.cursor_visible = true;
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        replacement_range: Option<Range<usize>>,
        text: &str,
        new_marked_text_range: Option<Range<usize>>,
        _window: &mut Window,
        _app: &mut App,
    ) {
        // [新增逻辑] 冲突检测与回滚
        // 如果没有指定替换范围（说明是新开始输入），且 marked_range 也是空的
        if replacement_range.is_none() && self.marked_range.is_none() {
            // 检查光标前一个字符是否和输入法想插入的 text 一样？
            // 例如：handle_keydown 插入了 "n"，现在 IME 也想插入 "n"
            if self.cursor_offset > 0 {
                let prev_char_len = 1; // 假设是英文，长度为1
                let start_check = self.cursor_offset - prev_char_len;
                let end_check = self.cursor_offset;

                let prev_text = self.text.slice(start_check..end_check).to_string();

                // 如果发现重复（比如之前已经插入了 'n'），就把它删掉！
                if prev_text == text {
                    self.text.remove(start_check..end_check);
                    self.cursor_offset -= prev_char_len;
                }
            }
        }

        // --- 下面保持你原有的逻辑不变 ---

        let range = replacement_range
            .or(self.marked_range.clone())
            .unwrap_or(self.cursor_offset..self.cursor_offset);

        let start = range.start.min(self.text.len_chars());
        let end = range.end.min(self.text.len_chars());

        if start < end {
            self.text.remove(start..end);
        }
        self.text.insert(start, text);

        if let Some(new_range) = new_marked_text_range {
            self.marked_range = Some((start + new_range.start)..(start + new_range.end));
        } else {
            self.marked_range = Some(start..(start + text.chars().count()));
        }

        self.cursor_offset = start + text.chars().count();
        self.last_interaction = Instant::now();
        self.cursor_visible = true;
    }

    fn unmark_text(&mut self, _window: &mut Window, _app: &mut App) {
        self.marked_range = None;
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        _window: &mut Window,
        _app: &mut App,
    ) -> Option<Bounds<Pixels>> {
        // 保留你的计算逻辑
        let (cursor_row, _cursor_col) = self.cursor_position();
        let top = 16.0 + (cursor_row as f32) * 28.0;
        let left = 100.0; // 暂时写死

        Some(Bounds {
            origin: Point {
                x: px(left),
                y: px(top),
            },
            size: Size {
                width: px(0.0),
                height: px(28.0),
            },
        })
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _app: &mut App,
    ) -> Option<usize> {
        Some(self.cursor_offset)
    }
}
