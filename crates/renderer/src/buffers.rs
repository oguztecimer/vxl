use ash::vk;
use ash::vk::{BufferCreateFlags, BufferUsageFlags, DeviceSize};
use vk_mem::{Alloc, AllocationCreateInfo, Allocator, MemoryUsage};

pub struct Buffers {

}

pub struct AllocatedBuffer {
    buffer: vk::Buffer,
    allocation: vk_mem::Allocation,
    allocation_info: vk_mem::AllocationInfo,
}

impl Buffers {
    pub fn new(allocator: Allocator,size:DeviceSize,usage_flags:BufferUsageFlags,memory_usage:MemoryUsage) -> Self{
        let buffer_create_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage_flags);

        let mut allocation_create_info = AllocationCreateInfo::default();
        allocation_create_info.usage = memory_usage;
        allocation_create_info.flags = BufferCreateFlags::default();
        unsafe{allocator.create_buffer(&buffer_create_info,&allocation_create_info)}.expect("Failed to create buffer");

        Self{

        }
    }
}


