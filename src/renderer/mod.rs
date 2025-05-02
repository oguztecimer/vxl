mod commands;
mod device;
pub mod images;
mod instance;
mod surface;
mod swapchain;

use crate::renderer::commands::Commands;
use crate::renderer::swapchain::*;
use ash::Entry;
use winit::window::Window;

pub struct Renderer {
    pub instance: instance::Instance,
    pub surface: surface::Surface,
    pub device: device::Device,
    pub swapchain: Swapchain,
    pub commands: Commands,
}

impl Renderer {
    pub fn new(window: &Window) -> Renderer {
        let entry = Entry::linked();
        let instance = instance::Instance::new(window, &entry);
        let surface = surface::Surface::new(window, &entry, &instance.handle);
        let device = device::Device::new(&instance.handle, &surface);
        let swapchain = Swapchain::new(&instance.handle, &device, &surface);
        let commands = Commands::new(&device.logical, device.queues.graphics.0);

        Renderer {
            instance,
            surface,
            device,
            swapchain,
            commands,
        }
    }

    pub fn recreate_swap_chain(&mut self) {
        unsafe {
            self.device
                .logical
                .device_wait_idle()
                .expect("Could not wait device idle");
        }
        self.swapchain.cleanup(&self.device.logical);
        self.swapchain = Swapchain::new(&self.instance.handle, &self.device, &self.surface);
    }

    pub fn cleanup(&self) {
        unsafe {
            self.device
                .logical
                .device_wait_idle()
                .expect("Could not wait device idle");
        }
        self.swapchain.cleanup(&self.device.logical);
        self.commands.cleanup(&self.device.logical);
        self.device.cleanup();
        self.surface.cleanup();
        self.instance.cleanup();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.cleanup()
    }
}
