use ash::vk::SurfaceKHR;
use ash::{Entry, Instance};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

pub struct Surface {
    pub(crate) handle: SurfaceKHR,
    pub(crate) loader: ash::khr::surface::Instance,
}

impl Surface {
    pub fn new(window: &Window, entry: &Entry, instance: &Instance) -> Surface {
        let display_handle = window
            .display_handle()
            .expect("Can't get raw display handle")
            .as_raw();
        let window_handle = window
            .window_handle()
            .expect("Can't get window handle")
            .as_raw();
        Surface {
            handle: unsafe {
                ash_window::create_surface(entry, instance, display_handle, window_handle, None)
            }
            .expect("Could not create surface"),
            loader: ash::khr::surface::Instance::new(entry, instance),
        }
    }
    pub fn cleanup(&self) {
        unsafe { self.loader.destroy_surface(self.handle, None) };
    }
}
