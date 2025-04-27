use ash::Device;
use ash::vk::{CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel, CommandPool};

pub fn create_command_buffer(command_pool: &CommandPool, logical_device: &Device) -> CommandBuffer {
    let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
        .command_pool(*command_pool)
        .command_buffer_count(1)
        .level(CommandBufferLevel::PRIMARY);

    unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }
        .expect("Could not allocate command buffers")[0]
}
