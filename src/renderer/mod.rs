mod commands;
mod descriptors;
mod device;
pub mod images;
mod instance;
mod surface;
mod swapchain;

use crate::renderer::commands::Commands;
use crate::renderer::descriptors::{DescriptionLayoutBuilder, DescriptorAllocator, Descriptors};
use crate::renderer::device::Device;
use crate::renderer::swapchain::*;
use ash::vk::{
    DescriptorImageInfo, DescriptorSetLayoutCreateFlags, DescriptorType, ImageLayout,
    ShaderStageFlags, WriteDescriptorSet,
};
use ash::{Entry, Instance};
use log::log;
use vk_mem::{Allocator, AllocatorCreateFlags, AllocatorCreateInfo};
use winit::window::Window;

pub struct Renderer {
    pub instance: instance::Instance,
    pub surface: surface::Surface,
    pub device: Device,
    pub allocator: Option<Allocator>,
    pub swapchain: Swapchain,
    pub commands: Commands,
    pub descriptors: Descriptors,
}

impl Renderer {
    pub fn new(window: &Window) -> Renderer {
        let entry = Entry::linked();
        let instance = instance::Instance::new(window, &entry);
        let surface = surface::Surface::new(window, &entry, &instance.handle);
        let device = device::Device::new(&instance.handle, &surface);
        let allocator = Self::create_allocator(&instance.handle, &device);
        let swapchain = Swapchain::new(&instance.handle, &device, &surface, &allocator);
        let commands = Commands::new(&device.logical, device.queues.graphics.0);
        let descriptors = Descriptors::new(&device.logical, swapchain.draw_image.image_view);
        Renderer {
            instance,
            surface,
            device,
            allocator: Some(allocator),
            swapchain,
            commands,
            descriptors,
        }
    }

    fn create_allocator(instance: &Instance, device: &Device) -> Allocator {
        let mut allocator_create_info =
            AllocatorCreateInfo::new(instance, &device.logical, device.physical);
        allocator_create_info.flags |= AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;
        unsafe { Allocator::new(allocator_create_info) }.expect("Could not create allocator")
    }

    pub fn recreate_swap_chain(&mut self) {
        unsafe {
            self.device
                .logical
                .device_wait_idle()
                .expect("Could not wait device idle");
        }
        self.swapchain
            .cleanup(&self.device.logical, self.allocator.as_ref().unwrap());
        self.swapchain = Swapchain::new(
            &self.instance.handle,
            &self.device,
            &self.surface,
            self.allocator.as_ref().unwrap(),
        );
    }

    pub fn cleanup(&mut self) {
        unsafe {
            self.device
                .logical
                .device_wait_idle()
                .expect("Could not wait device idle");
        }
        self.swapchain
            .cleanup(&self.device.logical, self.allocator.as_ref().unwrap());
        self.commands.cleanup(&self.device.logical);
        self.allocator.take();
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
