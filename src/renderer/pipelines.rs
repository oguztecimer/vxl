use crate::renderer::descriptors::Descriptors;
use ash::Device;
use ash::vk::{
    ComputePipelineCreateInfo, Pipeline, PipelineCache, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineShaderStageCreateInfo, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags,
};
use std::ffi::CString;
use vk_shader_macros::include_glsl;

const COMP: &[u32] = include_glsl!("resources/shaders/example/gradient.comp");
// const FRAG: &[u32] = include_glsl!("shaders/example.glsl", kind: frag);
// const RGEN: &[u32] = include_glsl!("shaders/example.rgen", target: vulkan1_2);

pub struct Pipelines {
    pub gradient_shader_module: ShaderModule,
    pub gradient_pipeline_layout: PipelineLayout,
    pub gradient_pipeline: Pipeline,
}

impl Pipelines {
    pub fn new(logical_device: &Device, descriptors: &Descriptors) -> Self {
        let layouts = [descriptors.draw_image_descriptor_layout];
        let pipeline_layout_create_info = PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        let gradient_pipeline_layout =
            unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }
                .expect("Could not create pipeline layout");

        let shader_module_create_info = ShaderModuleCreateInfo::default().code(COMP);
        let gradient_shader_module =
            unsafe { logical_device.create_shader_module(&shader_module_create_info, None) }
                .expect("Could not create shader module");
        let name = CString::new("main").expect("Could not create CString");
        let name = name.as_c_str();
        let shader_stage_create_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::COMPUTE)
            .name(name)
            .module(gradient_shader_module);
        let compute_pipeline_create_info = ComputePipelineCreateInfo::default()
            .stage(shader_stage_create_info)
            .layout(gradient_pipeline_layout);
        let create_infos = [compute_pipeline_create_info];
        let gradient_pipeline = unsafe {
            logical_device.create_compute_pipelines(PipelineCache::null(), &create_infos, None)
        }
        .expect("Could not create compute pipelines")[0];
        Self {
            gradient_pipeline_layout,
            gradient_pipeline,
            gradient_shader_module,
        }
    }

    pub fn cleanup(&mut self, logical_device: &Device) {
        unsafe {
            logical_device.destroy_pipeline(self.gradient_pipeline, None);
            logical_device.destroy_pipeline_layout(self.gradient_pipeline_layout, None);
            logical_device.destroy_shader_module(self.gradient_shader_module, None);
        }
    }
}
