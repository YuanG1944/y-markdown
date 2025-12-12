use gpui::*;
use y_markdown::ui::editor::EditorView;

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(400.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_win, cx| {
                cx.new(|cx| {
                    let view = EditorView::new(cx);
                    _win.focus(&view.focus_handle);
                    view
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
