use gpui::*;
use y_markdown::ui::editor::views::EditorView;

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(400.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |win, cx| {
                cx.new(|cx| {
                    let view = EditorView::new(cx);
                    win.focus(&view.focus_handle);
                    view
                })
            },
        )
        .unwrap();

        cx.activate(true);
    });
}
