use crate::descriptors::Descriptors;
use ash::Device;
use ash::vk::{
    ColorComponentFlags, ComputePipelineCreateInfo, DynamicState, Format,
    GraphicsPipelineCreateInfo, LogicOp, Pipeline, PipelineCache,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineRenderingCreateInfo, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, PushConstantRange,
    ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags,
};
use std::ffi::CString;
use vk_shader_macros::include_glsl;

const SIM: &[u32] = include_glsl!("../../resources/shaders/simulation.comp");
const VERT: &[u32] = include_glsl!("../../resources/shaders/fullScreen.vert");
const FRAG: &[u32] = include_glsl!("../../resources/shaders/fullScreen.frag");

#[repr(C)]
#[derive(Default)]
pub struct ComputePushConstants {
    pub swap_io: bool,
}

pub struct ComputePipeline {
    pub pipeline: Pipeline,
    pub pipeline_layout: PipelineLayout,
    pub shader_module: ShaderModule,
    pub data: ComputePushConstants,
}

pub struct GraphicsPipeline {
    pub pipeline: Pipeline,
    pub pipeline_layout: PipelineLayout,
    pub shader_modules: Vec<ShaderModule>,
}

pub struct Pipelines {
    pub simulation_pipeline: ComputePipeline,
    pub draw_pipeline: GraphicsPipeline,
}

impl GraphicsPipeline {
    pub fn new(logical_device: &Device, descriptors: &Descriptors) -> Self {
        let mut rendering_create_info = PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&[Format::R16G16B16A16_SFLOAT]) //DEFERRED ICIN BIRDEN FAZLA KOY!
            .depth_attachment_format(Format::UNDEFINED);

        let vertex_input_state_create_info = PipelineVertexInputStateCreateInfo::default();

        let viewport_state_create_info = PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let color_blend_attachment_states = [PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .color_write_mask(ColorComponentFlags::RGBA)];
        let color_blend_state_create_info = PipelineColorBlendStateCreateInfo::default()
            .logic_op(LogicOp::COPY)
            .logic_op_enable(false)
            .attachments(&color_blend_attachment_states);

        let dynamic_states = [DynamicState::VIEWPORT, DynamicState::SCISSOR];
        let dynamic_state_create_info =
            PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let mut shader_modules = Vec::new();
        let shader_module_create_info = ShaderModuleCreateInfo::default().code(VERT);
        let shader_module =
            unsafe { logical_device.create_shader_module(&shader_module_create_info, None) }
                .expect("Could not create shader module");
        shader_modules.push(shader_module);
        let shader_stage_name = CString::new("main").expect("Could not create CString");
        let shader_stage_name = shader_stage_name.as_c_str();
        let vert_stage_create_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::VERTEX)
            .name(shader_stage_name)
            .module(shader_module);

        let shader_module_create_info = ShaderModuleCreateInfo::default().code(FRAG);
        let shader_module =
            unsafe { logical_device.create_shader_module(&shader_module_create_info, None) }
                .expect("Could not create shader module");
        shader_modules.push(shader_module);
        let shader_stage_name = CString::new("main").expect("Could not create CString");
        let shader_stage_name = shader_stage_name.as_c_str();
        let frag_stage_create_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::FRAGMENT)
            .name(shader_stage_name)
            .module(shader_module);

        let stages = [vert_stage_create_info, frag_stage_create_info];

        let input_assembly_state_create_info = PipelineInputAssemblyStateCreateInfo::default()
            .topology(PrimitiveTopology::TRIANGLE_STRIP)
            .primitive_restart_enable(false);

        let rasterization_state_create_info = PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(PolygonMode::FILL)
            .cull_mode(ash::vk::CullModeFlags::NONE)
            .front_face(ash::vk::FrontFace::CLOCKWISE)
            .line_width(1.0);

        let multisample_state_create_info = PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1);

        let depth_stencil_state_create_info =
            PipelineDepthStencilStateCreateInfo::default().depth_test_enable(false);

        let layouts = [descriptors.full_screen_descriptor_layout];
        let pipeline_layout_create_info = PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        let pipeline_layout =
            unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None) }
                .expect("Could not create pipeline layout");

        let graphics_pipeline_create_info = GraphicsPipelineCreateInfo::default()
            .push_next(&mut rendering_create_info)
            .vertex_input_state(&vertex_input_state_create_info)
            .viewport_state(&viewport_state_create_info)
            .color_blend_state(&color_blend_state_create_info)
            .dynamic_state(&dynamic_state_create_info)
            .stages(&stages)
            .input_assembly_state(&input_assembly_state_create_info)
            //.tessellation_state()
            .rasterization_state(&rasterization_state_create_info)
            .multisample_state(&multisample_state_create_info)
            .depth_stencil_state(&depth_stencil_state_create_info)
            .layout(pipeline_layout);

        let graphics_pipeline_create_infos = [graphics_pipeline_create_info];
        let pipeline = unsafe {
            logical_device
                .create_graphics_pipelines(
                    PipelineCache::null(),
                    &graphics_pipeline_create_infos,
                    None,
                )
                .expect("Could not create graphics pipelines")
        }[0];

        Self {
            pipeline,
            pipeline_layout,
            shader_modules,
        }
    }
    pub fn cleanup(&self, logical_device: &Device) {
        unsafe {
            logical_device.destroy_pipeline(self.pipeline, None);
            logical_device.destroy_pipeline_layout(self.pipeline_layout, None);
            for shader_module in self.shader_modules.iter() {
                logical_device.destroy_shader_module(*shader_module, None);
            }
        }
    }
}

impl ComputePipeline {
    pub fn new(
        logical_device: &Device,
        descriptors: &Descriptors,
        data: ComputePushConstants,
    ) -> Self {
        let layouts = [descriptors.compute_descriptor_layout];
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
        let shader_module_create_info = ShaderModuleCreateInfo::default().code(SIM);
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
        let simulation_pipeline = ComputePipeline::new(
            logical_device,
            descriptors,
            ComputePushConstants { swap_io: false },
        );
        let draw_pipeline = GraphicsPipeline::new(logical_device, descriptors);
        Self {
            simulation_pipeline,
            draw_pipeline,
        }
    }

    pub fn cleanup(&self, logical_device: &Device) {
        self.draw_pipeline.cleanup(logical_device);
        self.simulation_pipeline.cleanup(logical_device);
    }
}

impl ComputePushConstants {
    pub fn swap(&mut self) {
        self.swap_io = !self.swap_io;
    }
}
