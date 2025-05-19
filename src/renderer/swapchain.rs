use crate::renderer::device::Device;
use crate::renderer::images::AllocatedImage;
use crate::renderer::surface::Surface;
use ash::vk::{
    CompositeAlphaFlagsKHR, Extent2D, Extent3D, Format, Image, ImageAspectFlags,
    ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType,
    PresentModeKHR, SharingMode, SwapchainCreateInfoKHR, SwapchainKHR,
};
use vk_mem::Allocator;

pub struct Swapchain {
    pub handle: SwapchainKHR,
    pub loader: ash::khr::swapchain::Device,
    pub image_views: Vec<ImageView>,
    pub images: Vec<Image>,
    pub extent: Extent2D,
    pub draw_image: AllocatedImage,
    pub depth_image: AllocatedImage,
}

impl Swapchain {
    pub fn new(
        instance: &ash::Instance,
        device: &Device,
        surface: &Surface,
        allocator: &Allocator,
    ) -> Self {
        let loader = ash::khr::swapchain::Device::new(instance, &device.logical);
        let surface_present_modes = unsafe {
            surface
                .loader
                .get_physical_device_surface_present_modes(device.physical, surface.handle)
        }
        .expect("Could not get surface present modes.");
        let surface_capabilities = unsafe {
            surface
                .loader
                .get_physical_device_surface_capabilities(device.physical, surface.handle)
        }
        .expect("Could not get surface capabilities");
        let surface_formats = unsafe {
            surface
                .loader
                .get_physical_device_surface_formats(device.physical, surface.handle)
        }
        .expect("Could not get surface formats");
        let surface_present_mode = surface_present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == PresentModeKHR::MAILBOX)
            .unwrap_or(PresentModeKHR::FIFO);
        let min_image_count =
            (surface_capabilities.min_image_count + 1).min(surface_capabilities.max_image_count);
        let image_format = surface_formats[0].format;
        let color_space = surface_formats[0].color_space;
        let extent = surface_capabilities.current_extent;
        let queue_family_indices_array = [device.queues.graphics.0];
        let create_info = SwapchainCreateInfoKHR::default()
            .surface(surface.handle)
            .min_image_count(min_image_count)
            .image_format(image_format)
            .image_color_space(color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(SharingMode::EXCLUSIVE)
            .queue_family_indices(&queue_family_indices_array)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(surface_present_mode);
        let handle = unsafe { loader.create_swapchain(&create_info, None) }
            .expect("Could not create swap chain!");
        let images = unsafe { loader.get_swapchain_images(handle) }
            .expect("Could not load swap chain images");
        let image_views: Vec<ImageView> = images
            .iter()
            .map(|&img| {
                let subresource_range = ImageSubresourceRange::default()
                    .aspect_mask(ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1);
                let info = ImageViewCreateInfo::default()
                    .subresource_range(subresource_range)
                    .image(img)
                    .view_type(ImageViewType::TYPE_2D)
                    .format(image_format);
                unsafe { device.logical.create_image_view(&info, None) }.unwrap()
            })
            .collect();
        let extent3d = Extent3D {
            width: surface_capabilities.current_extent.width,
            height: surface_capabilities.current_extent.height,
            depth: 1,
        };
        let draw_image = AllocatedImage::new(
            device,
            allocator,
            Format::R16G16B16A16_SFLOAT,
            extent3d,
            ImageUsageFlags::TRANSFER_SRC
                | ImageUsageFlags::TRANSFER_DST
                | ImageUsageFlags::STORAGE
                | ImageUsageFlags::COLOR_ATTACHMENT,
            ImageAspectFlags::COLOR,
        );
        let depth_image = AllocatedImage::new(
            device,
            allocator,
            Format::D32_SFLOAT,
            extent3d,
            ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            ImageAspectFlags::DEPTH,
        );
        Self {
            handle,
            loader,
            images,
            image_views,
            extent,
            draw_image,
            depth_image,
        }
    }

    pub fn cleanup(&mut self, logical_device: &ash::Device, allocator: &Allocator) {
        unsafe {
            self.draw_image.cleanup(logical_device, allocator);
            self.depth_image.cleanup(logical_device, allocator);
            for view in &self.image_views {
                logical_device.destroy_image_view(*view, None)
            }
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}
