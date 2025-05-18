use crate::renderer::buffers::AllocatedBuffer;
use crate::renderer::device::Device;
use crate::renderer::immediate_commands::ImmediateCommands;
use ash::vk::{BufferCopy, BufferDeviceAddressInfo, BufferUsageFlags, DeviceAddress, DeviceSize};
use glam::{Vec3, Vec4};
use std::ptr::copy_nonoverlapping;
use vk_mem::{Allocator, MemoryUsage};

#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec4,
}

pub struct GPUMeshBuffers {
    pub index_buffer: AllocatedBuffer,
    pub vertex_buffer: AllocatedBuffer,
    pub vertex_buffer_address: DeviceAddress,
}

pub fn upload_mesh(
    device: &Device,
    immediate_commands: &ImmediateCommands,
    allocator: &Allocator,
    indices: &[u32],
    vertices: &[Vertex],
) -> GPUMeshBuffers {
    let index_buffer_size = size_of_val(indices) as DeviceSize;
    let vertex_buffer_size = size_of_val(vertices) as DeviceSize;
    dbg!(index_buffer_size);
    dbg!(vertex_buffer_size);
    let index_buffer = AllocatedBuffer::new(
        allocator,
        index_buffer_size,
        BufferUsageFlags::INDEX_BUFFER | BufferUsageFlags::TRANSFER_DST,
        MemoryUsage::AutoPreferDevice,
    );
    let vertex_buffer = AllocatedBuffer::new(
        allocator,
        vertex_buffer_size,
        BufferUsageFlags::STORAGE_BUFFER
            | BufferUsageFlags::TRANSFER_DST
            | BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        MemoryUsage::AutoPreferDevice,
    );

    let device_address_info = BufferDeviceAddressInfo::default().buffer(vertex_buffer.buffer);
    let vertex_buffer_address = unsafe {
        device
            .logical
            .get_buffer_device_address(&device_address_info)
    };

    let staging_buffer = &mut AllocatedBuffer::new(
        allocator,
        index_buffer_size + vertex_buffer_size,
        BufferUsageFlags::TRANSFER_SRC,
        MemoryUsage::CpuOnly,
    );

    unsafe {
        let data = allocator
            .map_memory(&mut staging_buffer.allocation)
            .expect("Could not map memory");
        copy_nonoverlapping(
            vertices.as_ptr() as *const u8,
            data,
            vertex_buffer_size as usize,
        );
        copy_nonoverlapping(
            indices.as_ptr() as *const u8,
            data.add(vertex_buffer_size as usize),
            index_buffer_size as usize,
        );
        allocator.unmap_memory(&mut staging_buffer.allocation);
    }
    immediate_commands.submit(device, |command_buffer, logical_device| {
        let vertex_buffer_copy_regions = [BufferCopy::default()
            .src_offset(0)
            .dst_offset(0)
            .size(vertex_buffer_size)];
        unsafe {
            logical_device.cmd_copy_buffer(
                command_buffer,
                staging_buffer.buffer,
                vertex_buffer.buffer,
                &vertex_buffer_copy_regions,
            )
        };
        let index_buffer_copy_regions = [BufferCopy::default()
            .src_offset(0)
            .dst_offset(0)
            .size(index_buffer_size)];
        unsafe {
            logical_device.cmd_copy_buffer(
                command_buffer,
                staging_buffer.buffer,
                index_buffer.buffer,
                &index_buffer_copy_regions,
            )
        };
    });
    unsafe { allocator.destroy_buffer(staging_buffer.buffer, &mut staging_buffer.allocation) }
    GPUMeshBuffers {
        index_buffer,
        vertex_buffer,
        vertex_buffer_address,
    }
}

impl GPUMeshBuffers {
    pub fn test(
        device: &Device,
        immediate_commands: &ImmediateCommands,
        allocator: &Allocator,
    ) -> Self {
        let indices = [0, 1, 2, 2, 1, 3];
        let vertices = [
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.0),
                normal: Vec3::default(),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.0),
                normal: Vec3::default(),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.0),
                normal: Vec3::default(),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.0),
                normal: Vec3::default(),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
        ];
        upload_mesh(device, immediate_commands, allocator, &indices, &vertices)
    }
}
