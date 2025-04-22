use ash::Device;
use ash::vk::{Fence, FenceCreateInfo, Semaphore, SemaphoreCreateInfo};

pub struct Sync {
    pub image_available_semaphore: Semaphore,
    pub render_finished_semaphore: Semaphore,
    pub in_flight_fence: Fence,
}
impl Sync {
    pub fn new(logical_device: &Device) -> Sync {
        let semaphore_create_info = SemaphoreCreateInfo::default();
        let fence_create_info = FenceCreateInfo::default();
        let image_available_semaphore =
            unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }
                .expect("Could not create semaphore");
        let render_finished_semaphore =
            unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }
                .expect("Could not create semaphore");
        let in_flight_fence = unsafe { logical_device.create_fence(&fence_create_info, None) }
            .expect("Could not create fence");
        Sync {
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        }
    }

    pub fn cleanup(&self, logical_device: &Device) {
        unsafe { logical_device.destroy_fence(self.in_flight_fence, None) };
        unsafe { logical_device.destroy_semaphore(self.image_available_semaphore, None) };
        unsafe { logical_device.destroy_semaphore(self.render_finished_semaphore, None) };
    }
}