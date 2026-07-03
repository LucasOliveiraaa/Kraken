mod app;
mod ui;

use app::App;
use winit::event_loop::EventLoop;

fn main() {
    tracy_client::Client::start();
    let event_loop = EventLoop::new().unwrap();

    let mut application = App::new();
    event_loop.run_app(&mut application).unwrap();
}