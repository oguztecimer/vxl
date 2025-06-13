use crate::commands;
use crate::device::Device;
use ash::vk::{
    CommandBuffer, CommandBufferBeginInfo, CommandBufferResetFlags, CommandBufferSubmitInfo,
    CommandBufferUsageFlags, CommandPool, Fence, FenceCreateFlags, FenceCreateInfo, SubmitInfo2,
};

pub struct ImmediateCommands {
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,
    pub fence: Fence,
}

impl ImmediateCommands {
    pub fn new(device: &Device) -> Self {
        let command_pool =
            commands::Commands::create_command_pool(&device.logical, device.queues.graphics.0);
        let command_buffer =
            commands::Commands::create_command_buffer(&command_pool, &device.logical);
        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
        let fence = unsafe { device.logical.create_fence(&fence_create_info, None) }
            .expect("Could not create fence");
        Self {
            command_pool,
            command_buffer,
            fence,
        }
    }

    pub fn submit<F>(&self, device: &Device, function: F)
    where
        F: FnOnce(CommandBuffer, &ash::Device),
    {
        let fences = [self.fence];
        let command_buffer_begin_info =
            CommandBufferBeginInfo::default().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        let command_buffer_submit_infos =
            [CommandBufferSubmitInfo::default().command_buffer(self.command_buffer)];
        let submit_info2s =
            [SubmitInfo2::default().command_buffer_infos(&command_buffer_submit_infos)];
        unsafe {
            device
                .logical
                .reset_fences(&fences)
                .expect("Could not reset fences");
            device
                .logical
                .reset_command_buffer(self.command_buffer, CommandBufferResetFlags::empty())
                .expect("Could not reset command buffer");
            device
                .logical
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)
                .expect("Could not begin command buffer");
            function(self.command_buffer, &device.logical);
            device
                .logical
                .end_command_buffer(self.command_buffer)
                .expect("Could not end command buffer");
            device
                .logical_sync2
                .queue_submit2(device.queues.graphics.1, &submit_info2s, self.fence)
                .expect("Could not submit queue");
            device
                .logical
                .wait_for_fences(&fences, true, 9999999999)
                .expect("Could not wait for fence");
        }
    }

    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_command_pool(self.command_pool, None);
            logical_device.destroy_fence(self.fence, None);
        }
    }
}
