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
mod descriptor;

use std::ffi::c_void;
use std::fs::File;
use std::ptr;
use ash::vk::*;
use ash::{Entry};
use winit::window::Window;
use crate::renderer::buffers::{Buffers};
use crate::renderer::command_pools::CommandPools;
use crate::renderer::descriptor::Descriptor;
use crate::renderer::vertex::*;
use crate::renderer::swapchain::*;

pub struct Renderer {
    pub instance: instance::Instance,
    pub surface: surface::Surface,
    pub device: device::Device,
    pub swapchain: Swapchain,
    pub descriptor: Descriptor,
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
        let mut descriptor = Descriptor::new(&device);
        let pipeline = pipeline::Pipeline::new(&device,&swapchain,&descriptor);
        let command_pools = CommandPools::new(&device);

        //
        let decoder = png::Decoder::new(File::open("resources/textures/texture.png").unwrap());
        let mut reader = decoder.read_info()
            .expect("Could not read info");
        let tex_width = reader.info().width;
        let tex_height = reader.info().height;
        let mut pixels = vec![0u8; (tex_width * tex_height * 4) as usize];
        // Read the image data
        reader
            .next_frame(&mut pixels)
            .expect("Could not read png");

        let image_size = (tex_width * tex_height * 4) as DeviceSize;
        let (staging_buffer,staging_buffer_memory) = Buffers::create_buffer(
            &device,
            &instance.handle,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            image_size,
            SharingMode::EXCLUSIVE,
            &[]
        );
        let data = unsafe {
            device.logical.map_memory(
                staging_buffer_memory,
                0,
                image_size,
                MemoryMapFlags::empty()
            )
        }
            .expect("Could not map memory");
        unsafe {
            ptr::copy_nonoverlapping(
                pixels.as_ptr() as *const c_void,
                data,
                image_size as usize,
            );
        }
        unsafe { device.logical.unmap_memory(staging_buffer_memory) };
        reader.finish()
            .expect("Could not finish");

        //

        let image_create_info = ImageCreateInfo::default()
            .image_type(ImageType::TYPE_2D)
            .extent(Extent3D{
                width: tex_width,
                height: tex_height,
                depth: 1
            })
            .mip_levels(1)
            .array_layers(1)
            .format(Format::R8G8B8A8_SRGB)
            .tiling(ImageTiling::OPTIMAL)
            .initial_layout(ImageLayout::UNDEFINED)
            .usage(ImageUsageFlags::TRANSFER_DST | ImageUsageFlags::SAMPLED)
            .sharing_mode(SharingMode::EXCLUSIVE)
            .samples(SampleCountFlags::TYPE_1);
        let texture_image = unsafe{device.logical.create_image(&image_create_info,None)}
            .expect("Could not create image");
        //
        let command_buffer = Buffers::begin_command_buffer(&device,command_pools.transfer);

        Buffers::end_command_buffer(&device,command_buffer,command_pools.transfer);

        //
        let frame_buffers = frame_buffers::create_frame_buffers(&swapchain, pipeline.render_pass, &device.logical);
        let buffers = Buffers::new(&instance.handle,&device,&command_pools.transfer);
        descriptor.create_descriptor_sets(&device,&buffers);
        let command_buffer = command_buffers::create_command_buffer(&command_pools.graphics,&device.logical);
        let sync = sync::Sync::new(&device.logical);
        Renderer { instance, surface, device, swapchain,descriptor, pipeline, command_pools, frame_buffers, buffers, command_buffer, sync, }
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
        let buffers = &[self.buffers.combined_buffer];
        let vertex_offsets = &[0];

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
                buffers,
                vertex_offsets,
            );
            self.device.logical.cmd_bind_index_buffer(
                self.command_buffer
                ,self.buffers.combined_buffer,
                self.buffers.combined_buffer_indices_offset as DeviceSize,
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

        let descriptor_sets = [self.descriptor.sets.as_ref().unwrap()[image_index]];
        unsafe {
            self.device.logical.cmd_set_viewport(self.command_buffer, 0, &viewports);
            self.device.logical.cmd_set_scissor(self.command_buffer, 0, &scissors);
            self.device.logical.cmd_bind_descriptor_sets(self.command_buffer,PipelineBindPoint::GRAPHICS,self.pipeline.layout,0,&descriptor_sets,&[]);
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
        self.descriptor.cleanup(&self.device.logical);
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
