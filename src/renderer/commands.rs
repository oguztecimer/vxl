use ash::Device;
use ash::vk::{
    CommandBuffer, CommandBufferAllocateInfo, CommandBufferBeginInfo, CommandBufferLevel,
    CommandBufferUsageFlags, CommandPool, CommandPoolCreateFlags, CommandPoolCreateInfo, Fence,
    FenceCreateFlags, FenceCreateInfo, Semaphore, SemaphoreCreateInfo,
};

pub const FRAME_OVERLAP: usize = 2;
pub struct FrameData {
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,
    pub swapchain_semaphore: Semaphore,
    pub render_semaphore: Semaphore,
    pub render_fence: Fence,
}

pub struct Commands {
    frames: Vec<FrameData>,
    frame_number: usize,
}

impl Commands {
    pub fn new(logical_device: &Device, graphics_queue_family_index: u32) -> Self {
        let mut frames = Vec::with_capacity(FRAME_OVERLAP);
        for _ in 0..FRAME_OVERLAP {
            let command_pool =
                Self::create_command_pool(logical_device, graphics_queue_family_index);
            let command_buffer = Self::create_command_buffer(&command_pool, logical_device);
            let semaphore_create_info = SemaphoreCreateInfo::default();
            let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);
            let swapchain_semaphore =
                unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }
                    .expect("Could not create semaphore");
            let render_semaphore =
                unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }
                    .expect("Could not create semaphore");
            let render_fence = unsafe { logical_device.create_fence(&fence_create_info, None) }
                .expect("Could not create fence");

            let frame_data = FrameData {
                command_pool,
                command_buffer,
                swapchain_semaphore,
                render_semaphore,
                render_fence,
            };
            frames.push(frame_data)
        }
        Self {
            frames,
            frame_number: 0,
        }
    }

    pub fn increment_frame(&mut self) {
        self.frame_number += 1;
    }
    pub fn get_current_frame(&self) -> &FrameData {
        &self.frames[self.frame_number % FRAME_OVERLAP]
    }
    pub fn create_command_pool(logical_device: &Device, queue_family_index: u32) -> CommandPool {
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        unsafe { logical_device.create_command_pool(&command_pool_create_info, None) }
            .expect("Could not create command pool")
    }

    pub fn create_command_buffer(
        command_pool: &CommandPool,
        logical_device: &Device,
    ) -> CommandBuffer {
        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .command_pool(*command_pool)
            .command_buffer_count(1)
            .level(CommandBufferLevel::PRIMARY);

        unsafe { logical_device.allocate_command_buffers(&command_buffer_allocate_info) }
            .expect("Could not allocate command buffers")[0]
    }

    pub fn begin_command_buffer(&self, logical_device: &Device) {
        let command_buffer_begin_info =
            CommandBufferBeginInfo::default().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            logical_device.begin_command_buffer(
                self.get_current_frame().command_buffer,
                &command_buffer_begin_info,
            )
        }
        .expect("Could not begin recording the command buffer");
    }

    pub fn end_command_buffer(&self, logical_device: &Device) {
        unsafe { logical_device.end_command_buffer(self.get_current_frame().command_buffer) }
            .expect("Could not end command buffer");
    }

    pub fn cleanup(&self, logical_device: &Device) {
        unsafe {
            for frame_data in self.frames.iter() {
                logical_device.destroy_command_pool(frame_data.command_pool, None);
                logical_device.destroy_fence(frame_data.render_fence, None);
                logical_device.destroy_semaphore(frame_data.render_semaphore, None);
                logical_device.destroy_semaphore(frame_data.swapchain_semaphore, None);
            }
        }
    }
}
