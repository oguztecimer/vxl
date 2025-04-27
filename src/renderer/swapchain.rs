use crate::renderer::device::Device;
use crate::renderer::surface::Surface;
use ash::vk::{
    CompositeAlphaFlagsKHR, Extent2D, Format, ImageAspectFlags, ImageSubresourceRange,
    ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, PresentModeKHR, SharingMode,
    SwapchainCreateInfoKHR, SwapchainKHR,
};

pub struct Swapchain {
    pub handle: SwapchainKHR,
    pub loader: ash::khr::swapchain::Device,
    pub image_views: Vec<ImageView>,
    pub extent: Extent2D,
    pub image_format: Format,
}

impl Swapchain {
    pub fn new(instance: &ash::Instance, device: &Device, surface: &Surface) -> Self {
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
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
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
        Self {
            handle,
            loader,
            image_views,
            extent,
            image_format,
        }
    }

    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            for view in &self.image_views {
                logical_device.destroy_image_view(*view, None)
            }
            self.loader.destroy_swapchain(self.handle, None)
        }
    }
}
