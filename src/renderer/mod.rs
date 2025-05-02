mod commands;
mod descriptor;
mod device;
pub mod images;
mod instance;
mod pipeline;
mod surface;
mod swapchain;
mod vertex;

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

    // pub fn record_command_buffer(&self, image_index: usize) {
    //     let clear_values = [ClearValue {
    //         color: ClearColorValue {
    //             float32: [0.1, 0.1, 0.1, 1.0],
    //         },
    //     }];
    //     let render_pass_begin_info = RenderPassBeginInfo::default()
    //         .render_pass(self.pipeline.render_pass)
    //         .clear_values(&clear_values)
    //         .framebuffer(self.frame_buffers[image_index])
    //         .render_area(Rect2D {
    //             offset: Offset2D { x: 0, y: 0 },
    //             extent: self.swapchain.extent,
    //         });
    //     let buffers = &[self.buffers.combined_buffer];
    //     let vertex_offsets = &[0];
    //
    //     unsafe {
    //         self.device.logical.cmd_begin_render_pass(
    //             self.command_buffer,
    //             &render_pass_begin_info,
    //             SubpassContents::INLINE,
    //         );
    //         self.device.logical.cmd_bind_pipeline(
    //             self.command_buffer,
    //             PipelineBindPoint::GRAPHICS,
    //             self.pipeline.handle,
    //         );
    //         self.device.logical.cmd_bind_vertex_buffers(
    //             self.command_buffer,
    //             0,
    //             buffers,
    //             vertex_offsets,
    //         );
    //         self.device.logical.cmd_bind_index_buffer(
    //             self.command_buffer,
    //             self.buffers.combined_buffer,
    //             self.buffers.combined_buffer_indices_offset as DeviceSize,
    //             IndexType::UINT16,
    //         )
    //     }
    //
    //     let viewport = Viewport::default()
    //         .x(0.0)
    //         .y(0.0)
    //         .min_depth(0.0)
    //         .max_depth(0.0)
    //         .width(self.swapchain.extent.width as f32)
    //         .height(self.swapchain.extent.height as f32);
    //
    //     let scissor = Rect2D::default()
    //         .extent(self.swapchain.extent)
    //         .offset(Offset2D { x: 0, y: 0 });
    //
    //     let viewports = [viewport];
    //     let scissors = [scissor];
    //
    //     let descriptor_sets = [self.descriptor.sets.as_ref().unwrap()[image_index]];
    //     unsafe {
    //         self.device
    //             .logical
    //             .cmd_set_viewport(self.command_buffer, 0, &viewports);
    //         self.device
    //             .logical
    //             .cmd_set_scissor(self.command_buffer, 0, &scissors);
    //         self.device.logical.cmd_bind_descriptor_sets(
    //             self.command_buffer,
    //             PipelineBindPoint::GRAPHICS,
    //             self.pipeline.layout,
    //             0,
    //             &descriptor_sets,
    //             &[],
    //         );
    //         self.device.logical.cmd_draw_indexed(
    //             self.command_buffer,
    //             get_indices().len() as u32,
    //             1,
    //             0,
    //             0,
    //             0,
    //         );
    //         self.device.logical.cmd_end_render_pass(self.command_buffer);
    //         self.device
    //             .logical
    //             .end_command_buffer(self.command_buffer)
    //             .expect("Could not end recording command buffer");
    //     }
    // }

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
