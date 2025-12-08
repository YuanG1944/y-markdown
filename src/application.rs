use crate::constant::{HEIGHT, WIDTH};
use glutin::context::PossiblyCurrentContext;
use glutin::surface::{GlSurface, Surface as GlutinSurface, WindowSurface};
use skia_safe::font_style::{Slant, Weight, Width};
use skia_safe::gpu::DirectContext;
use skia_safe::{Color, Font, FontMgr, FontStyle, Paint, Point, Surface};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

const FAMILY_NAMES: [&str; 3] = ["PingFang SC", "Microsoft YaHei UI", "Noto Sans CJK SC"];

// Guarantee the drop order inside the FnMut closure. `Window` _must_ be dropped after
// `DirectContext`.
//
// <https://github.com/rust-skia/rust-skia/issues/476>
#[derive(Debug)]
pub(crate) struct Env {
    pub(crate) surface: Surface,
    pub(crate) gl_surface: GlutinSurface<WindowSurface>,
    pub(crate) gr_context: DirectContext,
    pub(crate) gl_context: PossiblyCurrentContext,
    pub(crate) window: Window,
}

#[derive(Default)]
pub(crate) struct Application {
    pub(crate) env: Option<Env>,
}

impl Application {
    pub fn new() -> Self {
        Self { env: None }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(self).expect("run() failed");
    }

    pub fn draw(&mut self) {
        if let Some(env) = &mut self.env {
            let canvas = env.surface.canvas();
            canvas.clear(Color::WHITE);

            canvas.save();

            let scale_factor = env.window.scale_factor() as f32;
            canvas.scale((scale_factor, scale_factor));

            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(Color::BLACK);
            let point = Point::new(WIDTH / 2.0, HEIGHT / 2.0);

            let font_style = FontStyle::new(Weight::NORMAL, Width::NORMAL, Slant::Upright);

            let mut typeface = None;
            for name in FAMILY_NAMES {
                if let Some(tf) = FontMgr::new().match_family_style(name, font_style) {
                    typeface = Some(tf);
                    break;
                }
            }

            let typeface = typeface.expect("Error: No fonts found at all.");

            let font = Font::new(typeface, 14.0);

            canvas.draw_str("Hello, world!", point, &font, &paint);

            canvas.restore();

            env.gr_context.flush_and_submit();
            env.gl_surface.swap_buffers(&env.gl_context).unwrap();
        }
    }
}
