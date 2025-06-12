// use crate::app::App;
// use winit::event_loop::{ControlFlow, EventLoop};

mod app;

fn main() {
    // let event_loop = EventLoop::new().unwrap();
    // event_loop.set_control_flow(ControlFlow::Poll);
    // let mut app = App::default();
    // event_loop
    //     .run_app(&mut app)
    //     .expect("Could not run event loop");
    renderer::app::App::run();
}
