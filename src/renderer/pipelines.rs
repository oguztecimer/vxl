use crate::renderer::descriptors::Descriptors;
use ash::Device;
use ash::vk::{
    ComputePipelineCreateInfo, Pipeline, PipelineCache, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineShaderStageCreateInfo, PushConstantRange, ShaderModule, ShaderModuleCreateInfo,
    ShaderStageFlags,
};
use glam::Vec4;
use std::ffi::CString;
use vk_shader_macros::include_glsl;

const COMP_SHADERS: [&[u32]; 2] = [
    include_glsl!("resources/shaders/example/gradient_color.comp"),
    include_glsl!("resources/shaders/example/gradient.comp"),
];
// const FRAG: &[u32] = include_glsl!("shaders/example.glsl", kind: frag);
// const RGEN: &[u32] = include_glsl!("shaders/example.rgen", target: vulkan1_2);

#[repr(C)]
#[derive(Default)]
pub struct ComputePushConstants {
    pub data1: Vec4,
    pub data2: Vec4,
    pub data3: Vec4,
    pub data4: Vec4,
}

pub struct ComputeEffect {
    pub name: String,
    pub pipeline: Pipeline,
    pub pipeline_layout: PipelineLayout,
    pub shader_module: ShaderModule,
    pub data: ComputePushConstants,
}

pub struct Pipelines {
    pub compute_effects: Vec<ComputeEffect>,
    pub active_compute_effect_index: usize,
}

impl ComputeEffect {
    pub fn new(
        logical_device: &Device,
        descriptors: &Descriptors,
        name: String,
        shader_index: usize,
        data: ComputePushConstants,
    ) -> Self {
        let layouts = [descriptors.draw_image_descriptor_layout];
        let push_constant_ranges = [PushConstantRange::default()
            .offset(0)
            .size(size_of::<ComputePushConstants>() as u32)
            .stage_flags(ShaderStageFlags::COMPUTE)];
        let pipeline_layout_create_info = PipelineLayoutCreateInfo::default()
            .set_layouts(&layouts)
            .push_constant_ranges(&push_constant_ranges);
        let pipeline_layout =
            unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }
                .expect("Could not create pipeline layout");
        let shader_module_create_info =
            ShaderModuleCreateInfo::default().code(COMP_SHADERS[shader_index]);
        let shader_module =
            unsafe { logical_device.create_shader_module(&shader_module_create_info, None) }
                .expect("Could not create shader module");
        let shader_stage_name = CString::new("main").expect("Could not create CString");
        let shader_stage_name = shader_stage_name.as_c_str();
        let shader_stage_create_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::COMPUTE)
            .name(shader_stage_name)
            .module(shader_module);
        let compute_pipeline_create_info = ComputePipelineCreateInfo::default()
            .stage(shader_stage_create_info)
            .layout(pipeline_layout);
        let create_infos = [compute_pipeline_create_info];
        let pipeline = unsafe {
            logical_device.create_compute_pipelines(PipelineCache::null(), &create_infos, None)
        }
        .expect("Could not create compute pipelines")[0];
        Self {
            name,
            pipeline,
            pipeline_layout,
            shader_module,
            data,
        }
    }

    pub fn cleanup(&self, logical_device: &Device) {
        unsafe {
            logical_device.destroy_pipeline(self.pipeline, None);
            logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
            logical_device.destroy_shader_module(self.shader_module, None);
        }
    }
}

impl Pipelines {
    pub fn new(logical_device: &Device, descriptors: &Descriptors) -> Self {
        let compute_effects: Vec<ComputeEffect> = vec![
            ComputeEffect::new(
                logical_device,
                descriptors,
                String::from("Deneme1"),
                1,
                ComputePushConstants {
                    data1: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    data2: Vec4::new(0.0, 0.0, 1.0, 1.0),
                    data3: Vec4::new(0.0, 1.0, 0.0, 1.0),
                    data4: Vec4::new(0.0, 0.0, 0.0, 1.0),
                },
            ),
            ComputeEffect::new(
                logical_device,
                descriptors,
                String::from("Deneme2"),
                0,
                ComputePushConstants {
                    data1: Vec4::new(1.0, 0.0, 0.0, 1.0),
                    data2: Vec4::new(0.0, 0.0, 1.0, 1.0),
                    data3: Vec4::new(0.0, 1.0, 0.0, 1.0),
                    data4: Vec4::new(0.0, 0.0, 0.0, 1.0),
                },
            ),
        ];
        Self {
            compute_effects,
            active_compute_effect_index: 0,
        }
    }

    pub fn get_current_effect(&self) -> &ComputeEffect {
        &self.compute_effects[self.active_compute_effect_index]
    }

    pub fn get_current_effect_mut(&mut self) -> &mut ComputeEffect {
        &mut self.compute_effects[self.active_compute_effect_index]
    }

    pub fn toggle_current_effect(&mut self) {
        self.active_compute_effect_index =
            (self.active_compute_effect_index + 1) % self.compute_effects.len();
    }

    pub fn cleanup(&self, logical_device: &Device) {
        for effect in self.compute_effects.iter() {
            effect.cleanup(logical_device);
        }
    }
}
