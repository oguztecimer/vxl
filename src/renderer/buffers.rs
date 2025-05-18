use ash::vk::{Buffer, BufferCreateInfo, BufferUsageFlags, DeviceSize, MemoryPropertyFlags};
use vk_mem::{
    Alloc, Allocation, AllocationCreateFlags, AllocationCreateInfo, Allocator, MemoryUsage,
};

pub struct AllocatedBuffer {
    pub buffer: Buffer,
    pub allocation: Allocation,
}

impl AllocatedBuffer {
    pub fn new(
        allocator: &Allocator,
        size: DeviceSize,
        usage_flags: BufferUsageFlags,
        memory_usage: MemoryUsage,
    ) -> Self {
        let buffer_create_info = BufferCreateInfo::default().size(size).usage(usage_flags);

        let allocation_create_info = AllocationCreateInfo {
            usage: memory_usage,
            preferred_flags: MemoryPropertyFlags::HOST_VISIBLE,
            flags: AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            ..Default::default()
        };
        let (buffer, allocation) =
            unsafe { allocator.create_buffer(&buffer_create_info, &allocation_create_info) }
                .expect("Could not create buffer");

        Self { buffer, allocation }
    }

    pub fn cleanup(&mut self, allocator: &Allocator) {
        unsafe {
            allocator.destroy_buffer(self.buffer, &mut self.allocation);
        }
    }
}
