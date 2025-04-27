use crate::renderer::buffers::{Buffers, UniformBufferObject};
use crate::renderer::device::Device;
use ash::vk::{
    DescriptorBufferInfo, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolSize,
    DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding,
    DescriptorSetLayoutCreateInfo, DescriptorType, DeviceSize, ShaderStageFlags,
    WriteDescriptorSet,
};

pub struct Descriptor {
    pub layout: DescriptorSetLayout,
    pub pool: Option<DescriptorPool>,
    pub sets: Option<Vec<DescriptorSet>>,
}

impl Descriptor {
    pub fn new(device: &Device) -> Self {
        let layout_binding = DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(ShaderStageFlags::VERTEX);
        let layout_bindings = [layout_binding];
        let layout_create_info =
            DescriptorSetLayoutCreateInfo::default().bindings(&layout_bindings);
        let layout = unsafe {
            device
                .logical
                .create_descriptor_set_layout(&layout_create_info, None)
        }
        .expect("Could not create Descriptor Set Layout");
        Self {
            layout,
            pool: None,
            sets: None,
        }
    }

    pub fn create_descriptor_sets(&mut self, device: &Device, buffers: &Buffers) {
        let buffer_count = crate::renderer::buffers::UNIFORM_BUFFER_COUNT as u32;
        let pool_size = DescriptorPoolSize::default()
            .descriptor_count(buffer_count)
            .ty(DescriptorType::UNIFORM_BUFFER);
        let pool_sizes = [pool_size];
        let descriptor_pool_create_info = DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(buffer_count);
        let pool = unsafe {
            device
                .logical
                .create_descriptor_pool(&descriptor_pool_create_info, None)
        }
        .expect("Could not create descriptor pool");

        let layouts = vec![self.layout; buffer_count as usize];
        let descriptor_set_allocate_info = DescriptorSetAllocateInfo::default()
            .set_layouts(&layouts)
            .descriptor_pool(pool);
        let descriptor_sets = unsafe {
            device
                .logical
                .allocate_descriptor_sets(&descriptor_set_allocate_info)
        }
        .expect("Could not allocate descriptor sets");
        for i in 0..buffer_count {
            let i = i as usize;
            let descriptor_buffer_info = DescriptorBufferInfo::default()
                .offset(0)
                .range(size_of::<UniformBufferObject>() as DeviceSize)
                .buffer(buffers.uniform_buffers[i]);
            let buffer_infos = [descriptor_buffer_info];
            let write_descriptor_set = WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .buffer_info(&buffer_infos);
            unsafe {
                device
                    .logical
                    .update_descriptor_sets(&[write_descriptor_set], &[])
            }
        }
        self.pool = Some(pool);
        self.sets = Some(descriptor_sets);
    }

    pub fn cleanup(&self, logical_device: &ash::Device) {
        if let Some(pool) = self.pool {
            unsafe { logical_device.destroy_descriptor_pool(pool, None) };
        }
        unsafe { logical_device.destroy_descriptor_set_layout(self.layout, None) };
    }
}
