use crate::renderer::surface::Surface;
use ash::Instance;
use ash::khr::copy_commands2::Device as CopyCommands2Device;
use ash::khr::dynamic_rendering::Device as DynamicRenderingDevice;
use ash::khr::synchronization2::Device as Sync2Device;
use ash::vk::{
    DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceVulkan12Features,
    PhysicalDeviceVulkan13Features, Queue, QueueFlags,
};

pub struct Device {
    pub physical: PhysicalDevice,
    pub logical: ash::Device,
    pub logical_sync2: Sync2Device,
    pub logical_copy_commands2: CopyCommands2Device,
    pub logical_dynamic_rendering: DynamicRenderingDevice,
    pub queues: Queues,
}

impl Device {
    pub fn new(instance: &Instance, surface: &Surface) -> Self {
        let physical_devices =
            unsafe { instance.enumerate_physical_devices() }.expect("Physical device error");
        if physical_devices.is_empty() {
            panic!("failed to find GPUs with Vulkan support!");
        }

        let mut selected_graphic_index = 0;
        let mut selected_transfer_index = 0;
        let mut selected_physical_device = None;
        let mut graphic_index = None;
        let mut found = false;

        for pd in physical_devices {
            graphic_index = None;
            let mut transfer_index = None;
            let queue_family_properties_in_all_devices =
                unsafe { instance.get_physical_device_queue_family_properties(pd) };
            'main_loop: for (i, queue_family_properties) in
                queue_family_properties_in_all_devices.iter().enumerate()
            {
                if queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
                {
                    if graphic_index.is_none() {
                        let supports_surface = unsafe {
                            surface.loader.get_physical_device_surface_support(
                                pd,
                                i as u32,
                                surface.handle,
                            )
                        }
                        .expect("Could not check if surface is supported");
                        if supports_surface {
                            graphic_index = Some(i);
                            selected_physical_device = Some(pd);
                        }
                    }
                } else if transfer_index.is_none()
                    && queue_family_properties
                        .queue_flags
                        .contains(QueueFlags::TRANSFER)
                {
                    transfer_index = Some(i);
                }
                if let (Some(graphic_index), Some(transfer_index)) = (graphic_index, transfer_index)
                {
                    selected_graphic_index = graphic_index as u32;
                    selected_transfer_index = transfer_index as u32;
                    found = true;
                    break 'main_loop;
                }
            }
        }
        if !found {
            if let Some(index) = graphic_index {
                selected_graphic_index = index as u32;
                selected_transfer_index = index as u32;
            } else {
                panic!("Physical device could not be found with the criteria");
            }
        }
        let physical = selected_physical_device.unwrap();

        let logical = Self::create_logical_device(
            selected_graphic_index,
            selected_transfer_index,
            instance,
            physical,
        );

        let logical_sync2 = Sync2Device::new(instance, &logical);
        let logical_copy_commands2 = CopyCommands2Device::new(instance, &logical);
        let logical_dynamic_rendering = DynamicRenderingDevice::new(instance, &logical);
        let graphics_queue = unsafe { logical.get_device_queue(selected_graphic_index, 0) };
        let queues = Queues {
            graphics: (selected_graphic_index, graphics_queue),
        };

        Self {
            physical,
            logical,
            logical_sync2,
            logical_copy_commands2,
            logical_dynamic_rendering,
            queues,
        }
    }

    fn create_logical_device(
        graphics_queue_family_index: u32,
        transfer_queue_family_index: u32,
        instance: &Instance,
        physical_device: PhysicalDevice,
    ) -> ash::Device {
        let device_queue_create_info_graphic = DeviceQueueCreateInfo::default()
            .queue_priorities(&[1.0])
            .queue_family_index(graphics_queue_family_index);

        let device_extension_names_raw = [
            ash::khr::swapchain::NAME.as_ptr(),
            ash::khr::synchronization2::NAME.as_ptr(),
            ash::khr::dynamic_rendering::NAME.as_ptr(),
            ash::khr::copy_commands2::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME.as_ptr(),
        ];
        let mut device_queue_create_info_vec = vec![device_queue_create_info_graphic];
        if graphics_queue_family_index != transfer_queue_family_index {
            let device_queue_create_info_transfer = DeviceQueueCreateInfo::default()
                .queue_priorities(&[1.0])
                .queue_family_index(transfer_queue_family_index);
            device_queue_create_info_vec.push(device_queue_create_info_transfer);
        }
        let mut vulkan13_features = PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);
        let mut vulkan12_features = PhysicalDeviceVulkan12Features::default()
            .buffer_device_address(true)
            .descriptor_indexing(true);
        let create_device_info = DeviceCreateInfo::default()
            .queue_create_infos(&device_queue_create_info_vec)
            .enabled_extension_names(&device_extension_names_raw)
            .push_next(&mut vulkan12_features)
            .push_next(&mut vulkan13_features);

        unsafe { instance.create_device(physical_device, &create_device_info, None) }
            .expect("Could not create logical device!")
    }
    pub fn cleanup(&self) {
        unsafe { self.logical.destroy_device(None) };
    }
}

#[derive(Debug)]
pub struct Queues {
    pub graphics: (u32, Queue),
}
