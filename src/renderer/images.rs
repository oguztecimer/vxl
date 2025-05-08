use crate::renderer::device::Device;
use ash::vk::{
    AccessFlags2, BlitImageInfo2, CommandBuffer, DependencyInfo, Extent2D, Extent3D, Filter,
    Format, Image, ImageAspectFlags, ImageBlit2, ImageCreateInfo, ImageLayout, ImageMemoryBarrier2,
    ImageSubresourceLayers, ImageSubresourceRange, ImageTiling, ImageType, ImageUsageFlags,
    ImageView, ImageViewCreateInfo, ImageViewType, MemoryPropertyFlags, PipelineStageFlags2,
    SampleCountFlags,
};
use vk_mem::{Alloc, Allocation, AllocationCreateInfo, Allocator, MemoryUsage};

pub struct AllocatedImage {
    pub image: Image,
    pub image_view: ImageView,
    //pub extent3d: Extent3D,
    //pub format: Format,
    pub allocation: Allocation,
}

impl AllocatedImage {
    pub fn new(
        device: &Device,
        allocator: &Allocator,
        format: Format,
        extent3d: Extent3D,
        usage_flags: ImageUsageFlags,
        aspect_flags: ImageAspectFlags,
    ) -> Self {
        let (image, allocation) = create_image(allocator, format, extent3d, usage_flags);
        let image_view = create_image_view(&device.logical, image, format, aspect_flags);

        Self {
            image,
            image_view,
            //extent3d,
            //format,
            allocation,
        }
    }
    pub fn cleanup(&mut self, logical_device: &ash::Device, allocator: &Allocator) {
        unsafe {
            logical_device.destroy_image_view(self.image_view, None);
            allocator.destroy_image(self.image, &mut self.allocation);
        }
    }
}

pub fn create_image(
    allocator: &Allocator,
    format: Format,
    extent: Extent3D,
    usage_flags: ImageUsageFlags,
) -> (Image, Allocation) {
    let info = ImageCreateInfo::default()
        .extent(extent)
        .format(format)
        .image_type(ImageType::TYPE_2D)
        .mip_levels(1)
        .array_layers(1)
        .samples(SampleCountFlags::TYPE_1)
        .tiling(ImageTiling::OPTIMAL)
        .usage(usage_flags);
    let allocation_create_info = AllocationCreateInfo {
        usage: MemoryUsage::AutoPreferDevice,
        preferred_flags: MemoryPropertyFlags::DEVICE_LOCAL,
        ..Default::default()
    };
    unsafe { allocator.create_image(&info, &allocation_create_info) }
        .expect("Could not create image")
}

pub fn create_image_view(
    logical_device: &ash::Device,
    image: Image,
    format: Format,
    aspect_flags: ImageAspectFlags,
) -> ImageView {
    let info = ImageViewCreateInfo::default()
        .format(format)
        .image(image)
        .view_type(ImageViewType::TYPE_2D)
        .subresource_range(
            ImageSubresourceRange::default()
                .layer_count(1)
                .level_count(1)
                .aspect_mask(aspect_flags),
        );
    unsafe { logical_device.create_image_view(&info, None) }.expect("Could not create image view")
}

pub fn transition_image_layout(
    device: &Device,
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
                })
                .level_count(1)
                .layer_count(1),
        )
        .image(image);
    let image_barriers = [image_barrier];
    let dependency_info = DependencyInfo::default().image_memory_barriers(&image_barriers);
    unsafe {
        device
            .logical_sync2
            .cmd_pipeline_barrier2(command_buffer, &dependency_info)
    }
}

pub fn copy_image_to_image(
    device: &Device,
    command_buffer: CommandBuffer,
    src_image: Image,
    dst_image: Image,
    src_size: Extent2D,
    dst_size: Extent2D,
) {
    let sub_resource = ImageSubresourceLayers::default()
        .aspect_mask(ImageAspectFlags::COLOR)
        .layer_count(1);
    let mut blit_region = ImageBlit2::default()
        .src_subresource(sub_resource)
        .dst_subresource(sub_resource);
    blit_region.src_offsets[1].x = src_size.width as i32;
    blit_region.src_offsets[1].y = src_size.height as i32;
    blit_region.src_offsets[1].z = 1;
    blit_region.dst_offsets[1].x = dst_size.width as i32;
    blit_region.dst_offsets[1].y = dst_size.height as i32;
    blit_region.dst_offsets[1].z = 1;
    let blit_regions = [blit_region];

    let blit_info = BlitImageInfo2::default()
        .filter(Filter::LINEAR)
        .regions(&blit_regions)
        .src_image(src_image)
        .src_image_layout(ImageLayout::TRANSFER_SRC_OPTIMAL)
        .dst_image(dst_image)
        .dst_image_layout(ImageLayout::TRANSFER_DST_OPTIMAL);

    unsafe {
        device
            .logical_copy_commands2
            .cmd_blit_image2(command_buffer, &blit_info)
    }
}
