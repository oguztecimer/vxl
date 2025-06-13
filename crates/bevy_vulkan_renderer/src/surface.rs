use ash::vk::SurfaceKHR;
use ash::{Entry, Instance};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub struct Surface {
    pub(crate) handle: SurfaceKHR,
    pub(crate) loader: ash::khr::surface::Instance,
}

impl Surface {
    pub fn new(
        raw_display_handle: RawDisplayHandle,
        raw_window_handle: RawWindowHandle,
        entry: &Entry,
        instance: &Instance,
    ) -> Surface {
        Surface {
            handle: unsafe {
                ash_window::create_surface(
                    entry,
                    instance,
                    raw_display_handle,
                    raw_window_handle,
                    None,
                )
            }
            .expect("Could not create surface"),
            loader: ash::khr::surface::Instance::new(entry, instance),
        }
    }
    pub fn cleanup(&self) {
        unsafe { self.loader.destroy_surface(self.handle, None) };
    }
}
