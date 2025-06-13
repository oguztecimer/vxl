use crate::buffers::{Buffers, MaterialProperties};
use crate::swapchain::Swapchain;
use ash::Device;
use ash::vk::{
    DescriptorBufferInfo, DescriptorImageInfo, DescriptorPool, DescriptorPoolCreateFlags,
    DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo,
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags,
    DescriptorSetLayoutCreateInfo, DescriptorType, ImageLayout, Sampler, SamplerCreateInfo,
    ShaderStageFlags, WriteDescriptorSet,
};

pub struct Descriptors {
    pub global_descriptor_allocator: DescriptorAllocator,
    pub simulation_descriptor_layout: DescriptorSetLayout,
    pub simulation_descriptor_set: DescriptorSet,
    pub render_descriptor_layout: DescriptorSetLayout,
    pub render_descriptor_set: DescriptorSet,
    pub map_editor_descriptor_layout: DescriptorSetLayout,
    pub map_editor_descriptor_set: DescriptorSet,
    pub sampler: Sampler,
}

pub struct DescriptorLayoutBuilder<'a> {
    bindings: Vec<DescriptorSetLayoutBinding<'a>>,
}

impl Descriptors {
    pub fn new(logical_device: &Device, swapchain: &Swapchain, buffers: &Buffers) -> Self {
        let sizes = [
            (DescriptorType::STORAGE_IMAGE, 1.0),
            (DescriptorType::COMBINED_IMAGE_SAMPLER, 1.0),
            (DescriptorType::UNIFORM_BUFFER, 1.0),
        ];
        let global_descriptor_allocator =
            DescriptorAllocator::new(logical_device, 20, Vec::from(sizes));

        let mut map_editor_descriptor_layout_builder = DescriptorLayoutBuilder::new();
        map_editor_descriptor_layout_builder.add_binding(
            0,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );

        let mut simulation_descriptor_layout_builder = DescriptorLayoutBuilder::new();
        simulation_descriptor_layout_builder.add_binding(
            0,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );
        simulation_descriptor_layout_builder.add_binding(
            1,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );
        simulation_descriptor_layout_builder.add_binding(
            2,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );
        simulation_descriptor_layout_builder.add_binding(
            3,
            DescriptorType::UNIFORM_BUFFER,
            ShaderStageFlags::COMPUTE,
        );

        let mut render_descriptor_layout_builder = DescriptorLayoutBuilder::new();
        render_descriptor_layout_builder.add_binding(
            0,
            DescriptorType::COMBINED_IMAGE_SAMPLER,
            ShaderStageFlags::FRAGMENT,
        );
        render_descriptor_layout_builder.add_binding(
            1,
            DescriptorType::COMBINED_IMAGE_SAMPLER,
            ShaderStageFlags::FRAGMENT,
        );
        let map_editor_descriptor_layout = map_editor_descriptor_layout_builder
            .get_layout(logical_device, DescriptorSetLayoutCreateFlags::default());
        let map_editor_descriptor_set =
            global_descriptor_allocator.allocate(logical_device, map_editor_descriptor_layout);
        let simulation_descriptor_layout = simulation_descriptor_layout_builder
            .get_layout(logical_device, DescriptorSetLayoutCreateFlags::default());
        let simulation_descriptor_set =
            global_descriptor_allocator.allocate(logical_device, simulation_descriptor_layout);
        let render_descriptor_layout = render_descriptor_layout_builder
            .get_layout(logical_device, DescriptorSetLayoutCreateFlags::default());
        let render_descriptor_set =
            global_descriptor_allocator.allocate(logical_device, render_descriptor_layout);
        let sampler_create_info = SamplerCreateInfo::default()
            .mag_filter(ash::vk::Filter::NEAREST)
            .min_filter(ash::vk::Filter::NEAREST);
        let sampler = unsafe { logical_device.create_sampler(&sampler_create_info, None) }
            .expect("Failed to create sampler");
        let result = Self {
            global_descriptor_allocator,
            simulation_descriptor_layout,
            simulation_descriptor_set,
            render_descriptor_layout,
            render_descriptor_set,
            map_editor_descriptor_layout,
            map_editor_descriptor_set,
            sampler,
        };
        result.update(logical_device, swapchain, buffers);
        result
    }

    fn update_map_editor(&self, logical_device: &Device, swapchain: &Swapchain) {
        let image_infos_0 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.simulation_image1.image_view)];
        let draw_image_writes = [WriteDescriptorSet::default()
            .dst_binding(0)
            .dst_set(self.map_editor_descriptor_set)
            .descriptor_count(1)
            .descriptor_type(DescriptorType::STORAGE_IMAGE)
            .image_info(&image_infos_0)];
        unsafe { logical_device.update_descriptor_sets(&draw_image_writes, &[]) }
    }

    fn update_simulation(&self, logical_device: &Device, swapchain: &Swapchain, buffers: &Buffers) {
        let image_infos_0 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.simulation_image0.image_view)];

        let image_infos_1 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.simulation_image1.image_view)];

        let image_infos_2 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.simulation_image2.image_view)];

        let buffer_infos_0 = [DescriptorBufferInfo::default()
            .buffer(buffers.material_properties_buffer.buffer)
            .offset(0)
            .range(size_of::<[MaterialProperties; 256]>() as u64)];

        let draw_image_writes = [
            WriteDescriptorSet::default()
                .dst_binding(0)
                .dst_set(self.simulation_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE)
                .image_info(&image_infos_0),
            WriteDescriptorSet::default()
                .dst_binding(1)
                .dst_set(self.simulation_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE)
                .image_info(&image_infos_1),
            WriteDescriptorSet::default()
                .dst_binding(2)
                .dst_set(self.simulation_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE)
                .image_info(&image_infos_2),
            WriteDescriptorSet::default()
                .dst_binding(3)
                .dst_set(self.simulation_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos_0),
        ];
        unsafe { logical_device.update_descriptor_sets(&draw_image_writes, &[]) }
    }

    fn update_render(&self, logical_device: &Device, swapchain: &Swapchain) {
        let image_infos0 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(swapchain.simulation_image1.image_view)
            .sampler(self.sampler)];
        let image_infos1 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(swapchain.simulation_image2.image_view)
            .sampler(self.sampler)];

        let draw_image_writes = [
            WriteDescriptorSet::default()
                .dst_binding(0)
                .dst_set(self.render_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_infos0),
            WriteDescriptorSet::default()
                .dst_binding(1)
                .dst_set(self.render_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&image_infos1),
        ];
        unsafe { logical_device.update_descriptor_sets(&draw_image_writes, &[]) }
    }

    pub fn update(&self, logical_device: &Device, swapchain: &Swapchain, buffers: &Buffers) {
        self.update_map_editor(logical_device, swapchain);
        self.update_simulation(logical_device, swapchain, buffers);
        self.update_render(logical_device, swapchain);
    }

    pub fn cleanup(&self, logical_device: &Device) {
        unsafe {
            logical_device.destroy_descriptor_set_layout(self.map_editor_descriptor_layout, None);
            logical_device.destroy_descriptor_set_layout(self.simulation_descriptor_layout, None);
            logical_device.destroy_descriptor_set_layout(self.render_descriptor_layout, None);
            logical_device.destroy_sampler(self.sampler, None);
            self.global_descriptor_allocator
                .destroy_pool(logical_device);
        }
    }
}

impl DescriptorLayoutBuilder<'_> {
    pub fn new() -> Self {
        Self { bindings: vec![] }
    }
    pub fn add_binding(
        &mut self,
        binding: u32,
        descriptor_type: DescriptorType,
        shader_stage_flags: ShaderStageFlags,
    ) {
        self.bindings.push(
            DescriptorSetLayoutBinding::default()
                .binding(binding)
                .descriptor_type(descriptor_type)
                .stage_flags(shader_stage_flags)
                .descriptor_count(1),
        );
    }

    pub fn get_layout(
        &self,
        logical_device: &Device,
        flags: DescriptorSetLayoutCreateFlags,
    ) -> DescriptorSetLayout {
        let info = DescriptorSetLayoutCreateInfo::default()
            .flags(flags)
            .bindings(&self.bindings);
        unsafe { logical_device.create_descriptor_set_layout(&info, None) }
            .expect("Could not create descriptor layout")
    }
}

pub struct DescriptorAllocator {
    pool: DescriptorPool,
}

impl DescriptorAllocator {
    pub fn new(
        logical_device: &Device,
        max_sets: u32,
        pool_ratios: Vec<(DescriptorType, f32)>,
    ) -> Self {
        let mut pool_sizes = vec![];
        for ratio in pool_ratios {
            pool_sizes.push(
                DescriptorPoolSize::default()
                    .descriptor_count((ratio.1 * max_sets as f32).floor() as u32)
                    .ty(ratio.0),
            )
        }
        let pool_create_info = DescriptorPoolCreateInfo::default()
            .max_sets(max_sets)
            .pool_sizes(&pool_sizes)
            .flags(DescriptorPoolCreateFlags::empty());
        let pool = unsafe { logical_device.create_descriptor_pool(&pool_create_info, None) }
            .expect("Could not create descriptor pool");
        Self { pool }
    }

    pub fn destroy_pool(&self, logical_device: &Device) {
        unsafe { logical_device.destroy_descriptor_pool(self.pool, None) };
    }

    pub fn allocate(
        &self,
        logical_device: &Device,
        descriptor_set_layout: DescriptorSetLayout,
    ) -> DescriptorSet {
        let layouts = [descriptor_set_layout];
        let allocate_info = DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts);

        unsafe { logical_device.allocate_descriptor_sets(&allocate_info) }
            .expect("Could not allocate info")[0]
    }
}
