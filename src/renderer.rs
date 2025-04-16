use ash::{Device, Entry, Instance, vk};
use ash::khr::surface;
use ash::vk::{ApplicationInfo, DeviceCreateInfo, DeviceQueueCreateInfo, InstanceCreateInfo, PhysicalDevice, SurfaceKHR};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

pub struct Renderer{
    entry: Entry,
    instance: Instance,
    surface: SurfaceKHR,
    physical_device: PhysicalDevice,
    logical_device: Device,
    queue_family_indices: QueueFamilyIndices
}

pub struct QueueFamilyIndices{
    graphics : u32,
}

impl Renderer{
    pub fn new(window: &Window) -> Renderer{

        unsafe {
            let entry = Entry::linked();
            let application_info =
                ApplicationInfo::default();

            let create_flags =
                if cfg!(any(target_os = "macos", target_os = "ios")) {
                    vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
                } else {
                    vk::InstanceCreateFlags::default()
                };
            let display_handle = window.display_handle().expect("Can't get raw display handle").as_raw();
            let window_handle = window.window_handle().expect("Can't get window handle").as_raw();

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

            let instance = entry.create_instance(&create_info,None).expect("Instance creation err");
            let surface = ash_window::create_surface(&entry,&instance,display_handle,window_handle,None).expect("Could not create surface");
            let surface_loader = surface::Instance::new(&entry,&instance);

            //Pick physical device
            //todo: physical devicei duzgun sec
            let physical_devices = instance.enumerate_physical_devices().expect("Physical device error");
            if physical_devices.len() == 0{
                panic!("failed to find GPUs with Vulkan support!");
            }
            //let physical_device = physical_devices[0];
            let (physical_device,queue_family_index) = physical_devices.iter()
                .find_map(|&pd| {
                    instance.get_physical_device_queue_family_properties(pd)
                        .iter()
                        .enumerate()
                        .find_map(|(index,&queue_family_properties)| {
                            let supports_graphic_and_surface =
                                queue_family_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface_loader
                                    .get_physical_device_surface_support(
                                        pd,
                                        index as u32,
                                        surface,
                                    )
                                    .unwrap();
                            if supports_graphic_and_surface {
                                Some((pd, index as u32))
                            } else {
                                None
                            }
                        })
                }).expect("Couldn't find suitable device");
            let queue_family_indices = QueueFamilyIndices{
                graphics : queue_family_index
            };
            //Create logical device
            //todo:queueyu duzgun sec
            let priorities = [1.0];
            let device_queue_create_info =
                DeviceQueueCreateInfo::default()
                .queue_priorities(&priorities)
                .queue_family_index(queue_family_index);
            let create_infos = [device_queue_create_info];
            let create_device_info= DeviceCreateInfo::default().queue_create_infos(&create_infos);
            let logical_device =
                instance.create_device
                (
                    physical_device,
                    &create_device_info,
                    None
                ).expect("Could not create logical device!");

            Renderer{
                entry,
                instance,
                surface,
                physical_device,
                logical_device,
                queue_family_indices
            }
        }
    }
}