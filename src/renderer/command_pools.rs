use crate::renderer::device::Device;
use ash::vk::{CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo};

pub struct CommandPools {
    pub graphics: CommandPool,
    pub transfer: CommandPool,
}

impl CommandPools {
    pub fn new(device: &Device) -> Self {
        let graphics = Self::create_command_pool(&device.logical, device.queues.graphics.0);
        let transfer = Self::create_command_pool(&device.logical, device.queues.transfer.0);
        Self { graphics, transfer }
    }

    fn create_command_pool(
        logical_device: &ash::Device,
        graphics_queue_family_index: u32,
    ) -> CommandPool {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(graphics_queue_family_index)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        unsafe { logical_device.create_command_pool(&command_pool_create_info, None) }
            .expect("Could not create command pool")
    }

    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_command_pool(self.graphics, None);
            logical_device.destroy_command_pool(self.transfer, None);
        }
    }
}
