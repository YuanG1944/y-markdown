// src/ui/editor/elements.rs
use super::EditorView;
use gpui::*;

pub struct CursorTracker {
    pub entity: Entity<EditorView>,
}

impl IntoElement for CursorTracker {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for CursorTracker {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _insp: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut s = Style::default();
        s.size.width = px(0.0).into();
        s.size.height = px(0.0).into();
        (window.request_layout(s, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Window,
        _: &mut App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        _: &mut Window,
        cx: &mut App,
    ) {
        self.entity.update(cx, |view, _cx| {
            // 注意：last_cursor_bounds 需要在 EditorView 中改为 pub(crate)
            view.last_cursor_bounds = Some(bounds);
        });
    }
}

pub struct InputBridge {
    pub entity: Entity<EditorView>,
    pub focus: FocusHandle,
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
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _insp: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut s = Style::default();
        s.size.width = px(0.0).into();
        s.size.height = px(0.0).into();
        (window.request_layout(s, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Window,
        _: &mut App,
    ) -> Self::PrepaintState {
        ()
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
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
