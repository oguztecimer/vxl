use ash::Device;
use ash::vk::{
    AccessFlags2, CommandBuffer, DependencyInfo, Image, ImageAspectFlags, ImageLayout,
    ImageMemoryBarrier2, ImageSubresourceRange, PipelineStageFlags2,
};

pub fn transition_image_layout(
    logical_device: &Device,
    command_buffer: CommandBuffer,
    image: Image,
    current_layout: ImageLayout,
    new_layout: ImageLayout,
) {
    let image_barrier = ImageMemoryBarrier2::default()
        .src_stage_mask(PipelineStageFlags2::ALL_COMMANDS)
        .src_access_mask(AccessFlags2::MEMORY_WRITE)
        .dst_stage_mask(PipelineStageFlags2::ALL_COMMANDS)
        .dst_access_mask(AccessFlags2::MEMORY_WRITE | AccessFlags2::MEMORY_READ)
        .old_layout(current_layout)
        .new_layout(new_layout)
        .subresource_range(
            ImageSubresourceRange::default()
                //.base_mip_level()
                .aspect_mask(if new_layout == ImageLayout::DEPTH_ATTACHMENT_OPTIMAL {
                    ImageAspectFlags::DEPTH
                } else {
                    ImageAspectFlags::COLOR
                }),
        )
        .image(image);
    let image_barriers = [image_barrier];
    let dependency_info = DependencyInfo::default().image_memory_barriers(&image_barriers);
    unsafe { logical_device.cmd_pipeline_barrier2(command_buffer, &dependency_info) }
}
