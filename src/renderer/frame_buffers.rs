use crate::renderer::swapchain::Swapchain;
use ash::Device;
use ash::vk::{Framebuffer, FramebufferCreateInfo, RenderPass};

pub fn create_frame_buffers(
    swapchain: &Swapchain,
    render_pass: RenderPass,
    logical_device: &Device,
) -> Vec<Framebuffer> {
    swapchain
        .image_views
        .iter()
        .map(|&image_view| {
            let image_view_array = [image_view];
            let frame_buffer_create_info = FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&image_view_array)
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);
            unsafe { logical_device.create_framebuffer(&frame_buffer_create_info, None) }
                .expect("Could not create frame buffer")
        })
        .collect()
}
