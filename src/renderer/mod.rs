mod vertex;
mod instance;
mod swapchain;
mod surface;
mod device;
mod pipeline;
mod buffers;
mod sync;
mod command_pools;
mod command_buffers;
mod frame_buffers;

use ash::vk::*;
use ash::{Entry};
use winit::window::Window;
use crate::renderer::buffers::Buffers;
use crate::renderer::command_pools::CommandPools;
use crate::renderer::vertex::*;
use crate::renderer::swapchain::*;

pub struct Renderer {
    pub instance: instance::Instance,
    pub surface: surface::Surface,
    pub device: device::Device,
    pub swapchain: Swapchain,
    pub pipeline: pipeline::Pipeline,
    pub command_pools: CommandPools,
    pub frame_buffers: Vec<Framebuffer>,
    pub buffers: Buffers,
    pub command_buffer: CommandBuffer,
    pub sync: sync::Sync,
}

impl Renderer {

    pub fn new(window: &Window) -> Renderer {
        let entry = Entry::linked();
        let instance = instance::Instance::new(window, &entry);
        let surface = surface::Surface::new(window, &entry, &instance.handle);
        let device = device::Device::new(&instance.handle,&surface);
        let swapchain = Swapchain::new(&instance.handle,&device,&surface);
        let pipeline = pipeline::Pipeline::new(&device,&swapchain);
        let command_pools = CommandPools::new(&device);
        let frame_buffers = frame_buffers::create_frame_buffers(&swapchain, pipeline.render_pass, &device.logical);
        let buffers = Buffers::new(&instance.handle,&device,&command_pools.transfer);
        let command_buffer = command_buffers::create_command_buffer(&command_pools.graphics,&device.logical);
        let sync = sync::Sync::new(&device.logical);
        Renderer { instance, surface, device, swapchain, pipeline, command_pools, frame_buffers, buffers, command_buffer, sync, }
    }

    pub fn record_command_buffer(&self, image_index: usize) {
        let command_buffer_begin_info = CommandBufferBeginInfo::default();
        unsafe { self.device.logical.begin_command_buffer(self.command_buffer, &command_buffer_begin_info) }
            .expect("Could not begin recording the command buffer");

        let clear_values = 
            [ClearValue { color: ClearColorValue { float32: [0.1, 0.1, 0.1, 1.0], } }];
        let render_pass_begin_info = RenderPassBeginInfo::default()
            .render_pass(self.pipeline.render_pass)
            .clear_values(&clear_values)
            .framebuffer(self.frame_buffers[image_index])
            .render_area(Rect2D {
                offset: Offset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            });
        let vertex_buffers = &[self.buffers.vertex_buffer];
        let offsets = &[0];
        
        unsafe {
            self.device.logical.cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                SubpassContents::INLINE,
            );
            self.device.logical.cmd_bind_pipeline(
                self.command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.pipeline.handle,
            );
            self.device.logical.cmd_bind_vertex_buffers(
                self.command_buffer,
                0,
                vertex_buffers,
                offsets,
            );
            self.device.logical.cmd_bind_index_buffer(
                self.command_buffer
                ,self.buffers.index_buffer,
                0,
                IndexType::UINT16
            )
        }

        let viewport = Viewport::default()
            .x(0.0)
            .y(0.0)
            .min_depth(0.0)
            .max_depth(0.0)
            .width(self.swapchain.extent.width as f32)
            .height(self.swapchain.extent.height as f32);

        let scissor = Rect2D::default()
            .extent(self.swapchain.extent)
            .offset(Offset2D { x: 0, y: 0 });

        let viewports = [viewport];
        let scissors = [scissor];

        unsafe {
            self.device.logical.cmd_set_viewport(self.command_buffer, 0, &viewports);
            self.device.logical.cmd_set_scissor(self.command_buffer, 0, &scissors);
            self.device.logical.cmd_draw_indexed(self.command_buffer, get_indices().len() as u32, 1, 0, 0,0);
            self.device.logical.cmd_end_render_pass(self.command_buffer);
            self.device.logical.end_command_buffer(self.command_buffer).expect("Could not end recording command buffer");
        }
    }
    
    pub fn recreate_swap_chain(&mut self) {
        unsafe { 
            self.device.logical.device_wait_idle().expect("Could not wait device idle");
            for fb in &self.frame_buffers { self.device.logical.destroy_framebuffer(*fb, None) }
        }
        self.swapchain.cleanup(&self.device.logical);
        self.swapchain = Swapchain::new(&self.instance.handle,&self.device,&self.surface);
        self.frame_buffers = frame_buffers::create_frame_buffers(&self.swapchain, self.pipeline.render_pass, &self.device.logical);
    }

    pub fn cleanup(&self) {
        unsafe { 
            self.device.logical.device_wait_idle().expect("Could not wait device idle");
            for fb in &self.frame_buffers { self.device.logical.destroy_framebuffer(*fb, None) }
        }
        self.swapchain.cleanup(&self.device.logical);
        self.buffers.cleanup(&self.device.logical);
        self.pipeline.cleanup(&self.device.logical);
        self.sync.cleanup(&self.device.logical);
        self.command_pools.cleanup(&self.device.logical);
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
