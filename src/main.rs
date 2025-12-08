use crate::application::Application;

mod application;
mod application_handler;
mod constant;

fn main() {
    let mut app = Application::new();
    app.run();
}
