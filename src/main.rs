mod app;
mod renderer;
mod utility;
mod world;

use crate::world::World;
use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let _world = World::new(4);
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop
        .run_app(&mut app)
        .expect("Could not run event loop");
}
