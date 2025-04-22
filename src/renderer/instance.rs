use ash::{vk, Entry};
use ash::vk::{ApplicationInfo, InstanceCreateInfo, API_VERSION_1_3};
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Window;

pub struct Instance{
    pub handle: ash::Instance
}
impl Instance{
    pub fn new(window: &Window, entry: &Entry) -> Instance {
        let application_info = ApplicationInfo::default().api_version(API_VERSION_1_3);

        let create_flags =
            if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };

        let display_handle = window
            .display_handle()
            .expect("Can't get raw display handle")
            .as_raw();

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        let mut extension_names = ash_window::enumerate_required_extensions(display_handle)
            .unwrap()
            .to_vec();
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        let extension_names = ash_window::enumerate_required_extensions(display_handle)
            .unwrap()
            .to_vec();

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
            // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
            extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }

        let create_info = InstanceCreateInfo::default()
            .application_info(&application_info)
            .flags(create_flags)
            .enabled_extension_names(&extension_names);
        Instance {
            handle: unsafe { entry
                .create_instance(&create_info, None)
                .expect("Instance creation err")
            }
        }
    }
    pub fn cleanup(&self){ unsafe{self.handle.destroy_instance(None);} }
}

