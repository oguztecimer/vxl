use crate::renderer::buffers::AllocatedBuffer;
use crate::renderer::device::Device;
use crate::renderer::immediate_commands::ImmediateCommands;
use ash::vk::{BufferCopy, BufferDeviceAddressInfo, BufferUsageFlags, DeviceAddress, DeviceSize};
use glam::{Vec3, Vec4};
use gltf::Gltf;
use std::ptr::copy_nonoverlapping;
use vk_mem::{Allocator, MemoryUsage};

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub color: Vec4,
}

#[derive(Debug)]
pub struct GPUMeshBuffers {
    pub index_count: usize,
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
        MemoryUsage::AutoPreferHost,
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
            .src_offset(vertex_buffer_size)
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
        index_count: indices.len(),
        index_buffer,
        vertex_buffer,
        vertex_buffer_address,
    }
}

impl GPUMeshBuffers {
    pub fn _test(
        device: &Device,
        immediate_commands: &ImmediateCommands,
        allocator: &Allocator,
    ) -> Self {
        let indices = [0, 1, 2, 2, 1, 3];
        let vertices = [
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.0),
                normal: Vec3::default(),
                color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.0),
                normal: Vec3::default(),
                color: Vec4::new(0.0, 1.0, 0.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.0),
                normal: Vec3::default(),
                color: Vec4::new(0.0, 0.0, 1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.0),
                normal: Vec3::default(),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            },
        ];
        upload_mesh(device, immediate_commands, allocator, &indices, &vertices)
    }

    pub fn load_from_glb(
        device: &Device,
        immediate_commands: &ImmediateCommands,
        allocator: &Allocator,
        path: &str,
        mesh_id: usize,
    ) -> Self {
        let gltf = Gltf::open(path).expect("Could not load gltf");
        let data = gltf.blob.as_ref().expect("No data inside blob").as_slice();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();
        let mesh = gltf.meshes().nth(mesh_id).expect("Could not find mesh");
        for primitive in mesh.primitives() {
            let vertex_offset = vertices.len() as u32;
            if let Some(indices_accessor) = primitive.indices() {
                let buffer_view = indices_accessor.view().expect("Could not access indices");
                let offset = buffer_view.offset() + indices_accessor.offset();
                let slice = data
                    .get(offset..offset + size_of::<u16>() * indices_accessor.count())
                    .expect("Could not read data");
                for chunk in slice.chunks_exact(2) {
                    let index = u16::from_le_bytes(chunk.try_into().expect("Could not read data"));
                    indices.push(vertex_offset + index as u32);
                }
            }
            let position_accessor = primitive
                .attributes()
                .find(|(sem, _)| sem == &gltf::Semantic::Positions)
                .expect("Missing POSITION attribute")
                .1;

            vertices.resize(
                vertices.len() + position_accessor.count(),
                Vertex::default(),
            );
            let buffer_view = position_accessor
                .view()
                .expect("Could not access positions");
            let offset = buffer_view.offset() + position_accessor.offset();
            let slice = data
                .get(offset..offset + size_of::<Vec3>() * position_accessor.count())
                .expect("Could not read data");
            for (i, chunk) in slice.chunks_exact(12).enumerate() {
                let x = f32::from_le_bytes(chunk[0..4].try_into().expect("Invalid position data"));
                let y = f32::from_le_bytes(chunk[4..8].try_into().expect("Invalid position data"));
                let z = f32::from_le_bytes(chunk[8..12].try_into().expect("Invalid position data"));
                vertices[vertex_offset as usize + i].position = Vec3::new(x, y, z);
            }

            let normal_accessor = primitive
                .attributes()
                .find(|(sem, _)| sem == &gltf::Semantic::Normals)
                .expect("Missing POSITION attribute")
                .1;

            let buffer_view = normal_accessor.view().expect("Could not access positions");
            let offset = buffer_view.offset() + normal_accessor.offset();
            let slice = data
                .get(offset..offset + size_of::<Vec3>() * normal_accessor.count())
                .expect("Could not read data");
            for (i, chunk) in slice.chunks_exact(12).enumerate() {
                let x = f32::from_le_bytes(chunk[0..4].try_into().expect("Invalid position data"));
                let y = f32::from_le_bytes(chunk[4..8].try_into().expect("Invalid position data"));
                let z = f32::from_le_bytes(chunk[8..12].try_into().expect("Invalid position data"));
                vertices[vertex_offset as usize + i].normal = Vec3::new(x, y, z);
                vertices[vertex_offset as usize + i].color = Vec4::new(x, y, z, 1.0);
            }
        }
        upload_mesh(device, immediate_commands, allocator, &indices, &vertices)
    }

    pub fn cleanup(&mut self, allocator: &Allocator) {
        self.index_buffer.cleanup(allocator);
        self.vertex_buffer.cleanup(allocator);
    }
}
