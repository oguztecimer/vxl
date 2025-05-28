use crate::swapchain::Swapchain;
use ash::Device;
use ash::vk::{
    DescriptorImageInfo, DescriptorPool, DescriptorPoolCreateFlags, DescriptorPoolCreateInfo,
    DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout,
    DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo,
    DescriptorType, ImageLayout, Sampler, SamplerCreateInfo, ShaderStageFlags, WriteDescriptorSet,
};

pub struct Descriptors {
    pub global_descriptor_allocator: DescriptorAllocator,
    pub compute_descriptor_layout: DescriptorSetLayout,
    pub compute_descriptor_set: DescriptorSet,
    pub full_screen_descriptor_layout: DescriptorSetLayout,
    pub full_screen_descriptor_set: DescriptorSet,
    pub sampler: Sampler,
}

pub struct DescriptorLayoutBuilder<'a> {
    bindings: Vec<DescriptorSetLayoutBinding<'a>>,
}

impl Descriptors {
    pub fn new(logical_device: &Device, swapchain: &Swapchain) -> Self {
        let sizes = [
            (DescriptorType::STORAGE_IMAGE, 1.0),
            (DescriptorType::COMBINED_IMAGE_SAMPLER, 1.0),
        ];
        let global_descriptor_allocator =
            DescriptorAllocator::new(logical_device, 20, Vec::from(sizes));

        let mut compute_descriptor_layout_builder = DescriptorLayoutBuilder::new();
        compute_descriptor_layout_builder.add_binding(
            0,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );
        compute_descriptor_layout_builder.add_binding(
            1,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );
        compute_descriptor_layout_builder.add_binding(
            2,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );
        compute_descriptor_layout_builder.add_binding(
            3,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );
        compute_descriptor_layout_builder.add_binding(
            4,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );

        let mut full_screen_descriptor_layout_builder = DescriptorLayoutBuilder::new();
        full_screen_descriptor_layout_builder.add_binding(
            0,
            DescriptorType::COMBINED_IMAGE_SAMPLER,
            ShaderStageFlags::FRAGMENT,
        );
        let compute_descriptor_layout = compute_descriptor_layout_builder
            .get_layout(logical_device, DescriptorSetLayoutCreateFlags::default());
        let compute_descriptor_set =
            global_descriptor_allocator.allocate(logical_device, compute_descriptor_layout);
        let full_screen_descriptor_layout = full_screen_descriptor_layout_builder
            .get_layout(logical_device, DescriptorSetLayoutCreateFlags::default());
        let full_screen_descriptor_set =
            global_descriptor_allocator.allocate(logical_device, full_screen_descriptor_layout);
        let sampler_create_info = SamplerCreateInfo::default()
            .mag_filter(ash::vk::Filter::NEAREST)
            .min_filter(ash::vk::Filter::NEAREST);
        let sampler = unsafe { logical_device.create_sampler(&sampler_create_info, None) }
            .expect("Failed to create sampler");
        let result = Self {
            global_descriptor_allocator,
            compute_descriptor_layout,
            compute_descriptor_set,
            full_screen_descriptor_layout,
            full_screen_descriptor_set,
            sampler,
        };
        result.update(logical_device, swapchain);
        result
    }

    fn update_compute(&self, logical_device: &Device, swapchain: &Swapchain) {
        let image_infos_0 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.compute_image.image_view)];

        let image_infos_1 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.properties1_in.image_view)];

        let image_infos_2 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.properties1_out.image_view)];

        let image_infos_3 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.properties2_in.image_view)];

        let image_infos_4 = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.properties2_out.image_view)];

        let draw_image_writes = [
            WriteDescriptorSet::default()
                .dst_binding(0)
                .dst_set(self.compute_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE)
                .image_info(&image_infos_0),
            WriteDescriptorSet::default()
                .dst_binding(1)
                .dst_set(self.compute_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE)
                .image_info(&image_infos_1),
            WriteDescriptorSet::default()
                .dst_binding(2)
                .dst_set(self.compute_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE)
                .image_info(&image_infos_2),
            WriteDescriptorSet::default()
                .dst_binding(3)
                .dst_set(self.compute_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE)
                .image_info(&image_infos_3),
            WriteDescriptorSet::default()
                .dst_binding(4)
                .dst_set(self.compute_descriptor_set)
                .descriptor_count(1)
                .descriptor_type(DescriptorType::STORAGE_IMAGE)
                .image_info(&image_infos_4),
        ];
        unsafe { logical_device.update_descriptor_sets(&draw_image_writes, &[]) }
    }

    fn update_full_screen(&self, logical_device: &Device, swapchain: &Swapchain) {
        let image_infos = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(swapchain.compute_image.image_view)
            .sampler(self.sampler)];
        let draw_image_writes = [WriteDescriptorSet::default()
            .dst_binding(0)
            .dst_set(self.full_screen_descriptor_set)
            .descriptor_count(1)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_infos)];
        unsafe { logical_device.update_descriptor_sets(&draw_image_writes, &[]) }
    }

    pub fn update(&self, logical_device: &Device, swapchain: &Swapchain) {
        self.update_compute(logical_device, swapchain);
        self.update_full_screen(logical_device, swapchain);
    }

    pub fn cleanup(&self, logical_device: &Device) {
        unsafe {
            logical_device.destroy_descriptor_set_layout(self.compute_descriptor_layout, None);
            logical_device.destroy_descriptor_set_layout(self.full_screen_descriptor_layout, None);
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
