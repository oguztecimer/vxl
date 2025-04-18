use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use crate::renderer::Renderer;

#[derive(Default)]
pub struct App{
    pub window: Option<Window>,
    pub renderer: Option<Renderer>
}

impl ApplicationHandler for App{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window
        (
            WindowAttributes::default()
                .with_title("vxl")
                //.with_fullscreen(Some(Fullscreen::Borderless()))
        ).unwrap();
        self.renderer = Some(Renderer::new(&window));
        self.window = Some(window);

    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
                self.renderer.as_ref().unwrap().cleanup();
            },
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            _=>()
        }
    }
}