use ash::{Device, Entry, Instance, vk};
use ash::khr::{surface, swapchain};
use ash::vk::{ApplicationInfo, CompositeAlphaFlagsKHR, DeviceCreateInfo, DeviceQueueCreateInfo, Extent2D, Format, Image, ImageAspectFlags, ImageSubresourceRange, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, InstanceCreateInfo, PhysicalDevice, PresentModeKHR, SharingMode, SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR};
use vk_shader_macros::include_glsl;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

const VERT:&[u32] = include_glsl!("shaders/shader.vert");
const FRAG:&[u32] = include_glsl!("shaders/shader.frag");

pub struct Renderer{
    entry: Entry,
    instance: Instance,
    surface: SurfaceKHR,
    surface_loader: surface::Instance,
    physical_device: PhysicalDevice,
    queue_family_indices: QueueFamilyIndices,
    logical_device: Device,
    swap_chain: SwapchainKHR,
    swap_chain_loader: swapchain::Device,
    swap_chain_images: Vec<Image>,
    swap_chain_image_views: Vec<ImageView>,
    swap_chain_image_format: Format,
    swap_chain_extent: Extent2D
}

pub struct QueueFamilyIndices{
    graphics : u32,
}

impl Renderer{
    fn create_instance(window: &Window,entry: &Entry) -> Instance{
        let application_info =
            ApplicationInfo::default();

        let create_flags =
            if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };
        let display_handle = window.display_handle().expect("Can't get raw display handle").as_raw();
        let mut extension_names =
            ash_window::enumerate_required_extensions(display_handle)
                .unwrap()
                .to_vec();
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
            // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
            extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }

        let create_info =
            InstanceCreateInfo::default()
                .application_info(&application_info)
                .flags(create_flags)
                .enabled_extension_names(&extension_names);

        unsafe{entry.create_instance(&create_info,None).expect("Instance creation err")}
    }
    fn create_physical_device_and_queue_family_index(
        instance: &Instance,
        surface_loader: &surface::Instance,
        surface: &SurfaceKHR
    ) -> (PhysicalDevice,u32){
        let physical_devices =
            unsafe{instance.enumerate_physical_devices()}
            .expect("Physical device error");
        if physical_devices.len() == 0{
            panic!("failed to find GPUs with Vulkan support!");
        }
        physical_devices.iter()
            .find_map(|&pd| {
                unsafe{instance.get_physical_device_queue_family_properties(pd)}
                    .iter()
                    .enumerate()
                    .find_map(|(index,&queue_family_properties)| {
                        let supports_graphic_and_surface =
                            queue_family_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && unsafe{surface_loader.get_physical_device_surface_support(
                                pd,
                                index as u32,
                                *surface,
                            )}
                                .unwrap();
                        if supports_graphic_and_surface {
                            Some((pd, index as u32))
                        } else {
                            None
                        }
                    })
            }).expect("Couldn't find suitable device")
    }

    fn create_logical_device(
        queue_family_index:u32,
        instance: &Instance,
        physical_device: &PhysicalDevice
    ) -> Device{
        let priorities = [1.0];
        let device_queue_create_info =
            DeviceQueueCreateInfo::default()
                .queue_priorities(&priorities)
                .queue_family_index(queue_family_index);
        let create_infos = [device_queue_create_info];
        let device_extension_names_raw = [
            swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
                ash::khr::portability_subset::NAME.as_ptr(),
        ];
        let create_device_info=
            DeviceCreateInfo::default()
                .queue_create_infos(&create_infos)
                .enabled_extension_names(&device_extension_names_raw);
        unsafe{instance.create_device
        (
            *physical_device,
            &create_device_info,
            None
        )}.expect("Could not create logical device!")
    }

    pub fn new(window: &Window) -> Renderer{
        let entry = Entry::linked();
        let instance = Self::create_instance(window,&entry);
        let display_handle = window
            .display_handle()
            .expect("Can't get raw display handle").as_raw();
        let window_handle = window.window_handle()
            .expect("Can't get window handle")
            .as_raw();
        let surface = unsafe{ash_window::create_surface(
            &entry,
            &instance,
            display_handle,
            window_handle,
            None
        )}.expect("Could not create surface");
        let surface_loader = surface::Instance::new(&entry,&instance);
        let (physical_device,queue_family_index) =
            Self::create_physical_device_and_queue_family_index(&instance,&surface_loader,&surface);

        let queue_family_indices = QueueFamilyIndices{
            graphics : queue_family_index
        };
        let logical_device =
            Self::create_logical_device(queue_family_index,&instance,&physical_device);


        let surface_present_modes = unsafe{surface_loader
            .get_physical_device_surface_present_modes(physical_device,surface)}
            .expect("Could not get surface present modes.");
        let surface_capabilities = unsafe{surface_loader
            .get_physical_device_surface_capabilities(physical_device,surface)}
            .expect("Could not get surface capabilities");
        let surface_formats = unsafe{surface_loader
            .get_physical_device_surface_formats(physical_device,surface)}
            .expect("Could not get surface formats");
        let surface_present_mode = surface_present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == PresentModeKHR::MAILBOX)
            .unwrap_or(PresentModeKHR::FIFO);

        let min_image_count =
            (surface_capabilities.min_image_count+1).min(surface_capabilities.max_image_count);

        let swap_chain_image_format = surface_formats[0].format;
        let swap_chain_color_space = surface_formats[0].color_space;
        let swap_chain_extent = surface_capabilities.current_extent;

        let swap_chain_loader = swapchain::Device::new(&instance,&logical_device);
        let indices = [queue_family_indices.graphics];
        let create_info =
            SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(min_image_count)
                .image_format(swap_chain_image_format)
                .image_color_space(swap_chain_color_space)
                .image_extent(swap_chain_extent)
                .image_array_layers(1)
                .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(SharingMode::EXCLUSIVE)
                .queue_family_indices(&indices)
                .pre_transform(surface_capabilities.current_transform)
                .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(surface_present_mode)
            ;
        let swap_chain =
            unsafe{swap_chain_loader.create_swapchain(&create_info,None)}
                .expect("Could not create swap chain!");

        let swap_chain_images =
            unsafe{swap_chain_loader.get_swapchain_images(swap_chain)}
            .expect("Could not load swap chain images");
        let swap_chain_image_views =
            swap_chain_images
                .iter()
                .map(|&img|{
                    let subresource_range =
                        ImageSubresourceRange::default()
                            .aspect_mask(ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1);
                    let info =
                        ImageViewCreateInfo::default()
                            .subresource_range(subresource_range)
                            .image(img)
                            .view_type(ImageViewType::TYPE_2D)
                            .format(swap_chain_image_format);
                    unsafe{logical_device.create_image_view(&info,None)}
                        .unwrap()
                }).collect();

        Renderer{
            entry,
            instance,
            surface,
            surface_loader,
            physical_device,
            queue_family_indices,
            logical_device,
            swap_chain,
            swap_chain_loader,
            swap_chain_images,
            swap_chain_image_views,
            swap_chain_image_format,
            swap_chain_extent
        }
    }

    pub fn cleanup(&self){
        unsafe {
            for view in &self.swap_chain_image_views{
                self.logical_device.destroy_image_view(*view,None);
            }
            self.swap_chain_loader.destroy_swapchain(self.swap_chain,None);
            self.surface_loader.destroy_surface(self.surface,None);
            self.instance.destroy_instance(None);
        }
    }
}