use ash::vk;
use ash::vk::{BufferCreateInfo, BufferUsageFlags, DeviceSize, MemoryPropertyFlags};
use vk_mem::{Alloc, AllocationCreateFlags, AllocationCreateInfo, Allocator, MemoryUsage};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MaterialProperties {
    pub friction: f32,
    pub sliding_friction: f32,
    pub acceleration: f32,
    pub reserved: f32,
}

pub struct Buffers {
    pub material_properties_buffer: AllocatedBuffer,
}

impl Buffers {
    pub fn new(allocator: &Allocator) -> Self {
        let size = size_of::<[MaterialProperties; 256]>() as DeviceSize;
        let mut material_properties_buffer = AllocatedBuffer::new(
            allocator,
            size,
            BufferUsageFlags::UNIFORM_BUFFER,
            MemoryUsage::Auto,
        );
        let material_properties = Self::create_material_properties();
        material_properties_buffer.write_data(allocator, &material_properties);

        Self {
            material_properties_buffer,
        }
    }

    fn create_material_properties() -> [MaterialProperties; 256] {
        let mut materials = [MaterialProperties::default(); 256];
        // Example: Initialize some materials
        materials[0] = MaterialProperties {
            friction: 0.0,
            sliding_friction: 0.0,
            acceleration: 0.0,
            reserved: 0.0,
        };
        materials[1] = MaterialProperties {
            friction: 0.001, // Empty material
            sliding_friction: 0.7,
            acceleration: 0.0,
            reserved: 0.0,
        };
        materials[2] = MaterialProperties {
            friction: 0.002,
            sliding_friction: 0.6,
            acceleration: 150.0,
            reserved: 0.0,
        };
        // Add more materials as needed
        materials
    }
    pub fn cleanup(&mut self, allocator: &Allocator) {
        self.material_properties_buffer.cleanup(allocator);
    }
}

pub struct AllocatedBuffer {
    pub buffer: vk::Buffer,
    pub allocation: vk_mem::Allocation,
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
            preferred_flags: MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            flags: AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            ..Default::default()
        };
        let (buffer, allocation) =
            unsafe { allocator.create_buffer(&buffer_create_info, &allocation_create_info) }
                .expect("Could not create buffer");

        Self { buffer, allocation }
    }

    pub fn write_data<T>(&mut self, allocator: &Allocator, data: &T) {
        let allocation_info = allocator.get_allocation_info(&self.allocation);
        let size = size_of::<T>() as DeviceSize;
        assert!(
            size <= allocation_info.size,
            "Data size exceeds buffer size"
        );

        let mapped_ptr = unsafe {
            allocator
                .map_memory(&mut self.allocation)
                .expect("Failed to map memory")
        };
        unsafe {
            std::ptr::copy_nonoverlapping(data as *const T as *const u8, mapped_ptr, size as usize);
        }
        unsafe { allocator.unmap_memory(&mut self.allocation) };
    }

    pub fn cleanup(&mut self, allocator: &Allocator) {
        unsafe {
            allocator.destroy_buffer(self.buffer, &mut self.allocation);
        }
    }
}
