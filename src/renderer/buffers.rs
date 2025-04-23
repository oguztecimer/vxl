use std::ptr;
use ash::Instance;
use ash::vk::*;
use crate::renderer::device::{Device};
use crate::renderer::vertex::{get_indices, get_vertices, Vertex};

pub struct Buffers{
    pub combined_buffer:Buffer,
    pub combined_buffer_memory: DeviceMemory,
    pub indices_offset:usize
}

impl Buffers {
    pub fn new(
        instance: &Instance,
        device: &Device,
        transfer_command_pool: &CommandPool

    ) -> Self{
        let (combined_buffer,combined_buffer_memory,indices_offset) = Self::create_combined_buffer(
            device,
            instance,
            transfer_command_pool
        );
        Self{
            combined_buffer,
            combined_buffer_memory,
            indices_offset
        }
    }

    fn create_combined_buffer(
        device: &Device,
        instance: &Instance,
        command_pool: &CommandPool
    ) -> (Buffer, DeviceMemory,usize) {
        //vertices
        let vertices = get_vertices();
        let vertices_size = size_of::<Vertex>() * vertices.len();
        let buffer_size = vertices_size;
        let aligned_buffer_size = (buffer_size + device.min_buffer_alignment -1) & !(device.min_buffer_alignment-1);

        let indices = get_indices();
        let index_size = size_of::<u16>();
        let indices_size =  index_size * indices.len();
        let buffer_size = aligned_buffer_size + indices_size;

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            device,
            instance,
            BufferUsageFlags::TRANSFER_SRC,
            MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT,
            buffer_size as DeviceSize,
            SharingMode::EXCLUSIVE,
            &[]
        );

        let data = unsafe {
            device.logical.map_memory(
                staging_buffer_memory,
                0,
                buffer_size as DeviceSize,
                MemoryMapFlags::empty(),
            )
        }
            .expect("Could not map memory");
        unsafe {
            ptr::copy_nonoverlapping(
                vertices.as_ptr() as *const std::ffi::c_void,
                data,
                vertices_size,
            );
        }
        let data = unsafe{(data as *mut u8).add(aligned_buffer_size) as *mut std::ffi::c_void};
        unsafe {
            ptr::copy_nonoverlapping(
                indices.as_ptr() as *const std::ffi::c_void,
                data,
                indices_size,
            );
        }

        unsafe { device.logical.unmap_memory(staging_buffer_memory) };
        let same_queues = device.queues.graphics.0 == device.queues.transfer.0;
        let (sharing_mode,concurrent_queue_family_indices) : (SharingMode,&[u32]) =
            if !same_queues {
                (SharingMode::CONCURRENT,&[device.queues.graphics.0,device.queues.transfer.0])
            }
            else{
                (SharingMode::EXCLUSIVE,&[])
            };
        let (buffer, buffer_memory) = Self::create_buffer(
            device,
            instance,
            BufferUsageFlags::VERTEX_BUFFER | BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST,
            MemoryPropertyFlags::DEVICE_LOCAL,
            buffer_size as DeviceSize,
            sharing_mode,
            concurrent_queue_family_indices
        );

        Self::copy_buffer(
            device,
            command_pool,
            staging_buffer,
            buffer,
            buffer_size as DeviceSize
        );
        unsafe { device.logical.destroy_buffer(staging_buffer, None) }
        unsafe { device.logical.free_memory(staging_buffer_memory, None) }
        (buffer, buffer_memory,aligned_buffer_size)

    }

    pub fn create_buffer(
        device: &Device,
        instance: &Instance,
        buffer_usage_flags: BufferUsageFlags,
        memory_property_flags: MemoryPropertyFlags,
        size: DeviceSize,
        sharing_mode: SharingMode,
        concurrent_queue_family_indices: &[u32]
    ) -> (Buffer, DeviceMemory) {
        //
        let mut buffer_create_info = BufferCreateInfo::default()
            .size(size)
            .usage(buffer_usage_flags)
            .sharing_mode(sharing_mode);

        buffer_create_info = {
            if sharing_mode == SharingMode::CONCURRENT {
                buffer_create_info.queue_family_indices(concurrent_queue_family_indices)
            }else{
                buffer_create_info
            }
        };
        let buffer = unsafe { device.logical.create_buffer(&buffer_create_info, None) }
            .expect("Could not create vertex buffer");
        let mem_requirements = unsafe { device.logical.get_buffer_memory_requirements(buffer) };
        let mem_properties = memory_property_flags;
        let memory_type_index = Self::find_memory_type_index(
            &device.physical,
            instance,
            mem_requirements.memory_type_bits,
            mem_properties,
        );
        let memory_allocate_info = MemoryAllocateInfo::default()
            .memory_type_index(memory_type_index)
            .allocation_size(mem_requirements.size);
        let buffer_memory = unsafe { device.logical.allocate_memory(&memory_allocate_info, None) }
            .expect("Could not allocate memory for vertex buffer");
        unsafe { device.logical.bind_buffer_memory(buffer, buffer_memory, 0) }
            .expect("Could not bind vertex buffer memory");
        (buffer, buffer_memory)
    }

    fn copy_buffer(
        device: &Device,
        command_pool: &CommandPool,
        src_buffer: Buffer,
        dst_buffer: Buffer,
        size: DeviceSize
    ) {
        let command_buffer_allocate_info = CommandBufferAllocateInfo::default()
            .level(CommandBufferLevel::PRIMARY)
            .command_pool(*command_pool)
            .command_buffer_count(1);
        let command_buffer =
            unsafe { device.logical.allocate_command_buffers(&command_buffer_allocate_info) }
                .expect("Could not allocate command buffers")[0];
        let command_buffer_begin_info =
            CommandBufferBeginInfo::default().flags(CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device.logical.begin_command_buffer(command_buffer, &command_buffer_begin_info) }
            .expect("Could not begin command buffer");
        let copy_regions = [BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size,
        }];
        unsafe { device.logical.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions) }
        unsafe { device.logical.end_command_buffer(command_buffer) }
            .expect("Could not end command buffer");
        let command_buffers = [command_buffer];
        let submit_info = SubmitInfo::default().command_buffers(&command_buffers);
        let submit_infos = [submit_info];
        unsafe {
            device.logical.queue_submit(device.queues.transfer.1, &submit_infos, Fence::null())
        }
            .expect("Could not submit queue");
        //todo: Improve parallelization: use fences
        unsafe { device.logical.queue_wait_idle(device.queues.transfer.1) }
            .expect("Could not wait for queue idle");
        unsafe { device.logical.free_command_buffers(*command_pool, &command_buffers) };
    }

    fn find_memory_type_index(
        physical_device: &PhysicalDevice,
        instance: &Instance,
        type_filter: u32,
        properties: MemoryPropertyFlags,
    ) -> u32 {
        let physical_device_memory_properties =
            unsafe { instance.get_physical_device_memory_properties(*physical_device) };
        for i in 0..physical_device_memory_properties.memory_type_count {
            if ((type_filter & (1 << i)) != 0)
                && ((physical_device_memory_properties.memory_types[i as usize].property_flags
                & properties)
                == properties)
            {
                return i;
            }
        }
        panic!("Could not find a suitable memory type");
    }
    
    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_buffer(self.combined_buffer, None);
            logical_device.free_memory(self.combined_buffer_memory, None);
        }
    }
}