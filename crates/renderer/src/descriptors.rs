use crate::swapchain::Swapchain;
use ash::Device;
use ash::vk::{DescriptorImageInfo, DescriptorPool, DescriptorPoolCreateFlags, DescriptorPoolCreateInfo, DescriptorPoolSize, DescriptorSet, DescriptorSetAllocateInfo, DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags, DescriptorSetLayoutCreateInfo, DescriptorType, ImageLayout, ImageView, ShaderStageFlags, WriteDescriptorSet};

pub struct Descriptors {
    pub global_descriptor_allocator: DescriptorAllocator,
    pub compute_descriptor_layout: DescriptorSetLayout,
    pub compute_descriptor_set: DescriptorSet,
    pub full_screen_descriptor_layout: DescriptorSetLayout,
    pub full_screen_descriptor_set: DescriptorSet,
}

pub struct DescriptorLayoutBuilder<'a> {
    bindings: Vec<DescriptorSetLayoutBinding<'a>>,
}

impl Descriptors {
    pub fn new(logical_device: &Device, swapchain: &Swapchain) -> Self {
        let sizes = [(DescriptorType::STORAGE_IMAGE, 1.0)];
        let global_descriptor_allocator =
            DescriptorAllocator::new(logical_device, 10, Vec::from(sizes));

        let mut compute_descriptor_layout_builder = DescriptorLayoutBuilder::new();
        compute_descriptor_layout_builder.add_binding(
            0,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::COMPUTE,
        );
        let compute_descriptor_layout = compute_descriptor_layout_builder
            .get_layout(logical_device, DescriptorSetLayoutCreateFlags::default());
        let compute_descriptor_set =
            global_descriptor_allocator.allocate(logical_device, compute_descriptor_layout);


        let mut full_screen_descriptor_layout_builder = DescriptorLayoutBuilder::new();
        full_screen_descriptor_layout_builder.add_binding(
            0,
            DescriptorType::STORAGE_IMAGE,
            ShaderStageFlags::FRAGMENT,
        );
        let full_screen_descriptor_layout = full_screen_descriptor_layout_builder
            .get_layout(logical_device, DescriptorSetLayoutCreateFlags::default());
        let full_screen_descriptor_set =
            global_descriptor_allocator.allocate(logical_device, full_screen_descriptor_layout);

        let result = Self {
            global_descriptor_allocator,
            compute_descriptor_layout,
            compute_descriptor_set,
            full_screen_descriptor_layout,
            full_screen_descriptor_set
        };
        result.update(logical_device, swapchain.draw_image.image_view);

        let image_infos = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(swapchain.draw_image.image_view)];
        let draw_image_writes = [WriteDescriptorSet::default()
            .dst_binding(0)
            .dst_set(full_screen_descriptor_set)
            .descriptor_count(1)
            .descriptor_type(DescriptorType::STORAGE_IMAGE)
            .image_info(&image_infos)];
        unsafe { logical_device.update_descriptor_sets(&draw_image_writes, &[]) }

        result
    }

    pub fn update(&self, logical_device: &Device, image_view: ImageView) {
        let image_infos = [DescriptorImageInfo::default()
            .image_layout(ImageLayout::GENERAL)
            .image_view(image_view)];
        let draw_image_writes = [WriteDescriptorSet::default()
            .dst_binding(0)
            .dst_set(self.compute_descriptor_set)
            .descriptor_count(1)
            .descriptor_type(DescriptorType::STORAGE_IMAGE)
            .image_info(&image_infos)];
        unsafe { logical_device.update_descriptor_sets(&draw_image_writes, &[]) }
    }

    pub fn cleanup(&self, logical_device: &Device) {
        unsafe {
            logical_device.destroy_descriptor_set_layout(self.compute_descriptor_layout, None);
            logical_device.destroy_descriptor_set_layout(self.full_screen_descriptor_layout, None);
            self.global_descriptor_allocator.destroy_pool(logical_device);
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

    // pub fn clear(&mut self) {
    //     self.bindings.clear();
    // }
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
