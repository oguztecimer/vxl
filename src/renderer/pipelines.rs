use crate::renderer::descriptors::Descriptors;
use ash::Device;
use ash::vk::{
    ColorComponentFlags, CompareOp, ComputePipelineCreateInfo, DeviceAddress, DynamicState, Format,
    GraphicsPipelineCreateInfo, LogicOp, Pipeline, PipelineCache,
    PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
    PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo,
    PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo,
    PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
    PipelineRenderingCreateInfo, PipelineShaderStageCreateInfo, PipelineVertexInputStateCreateInfo,
    PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, PushConstantRange,
    ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags,
};
use glam::{Mat4, Vec4};
use std::ffi::CString;
use vk_shader_macros::include_glsl;

const COMP_SHADERS: [&[u32]; 2] = [
    include_glsl!("resources/shaders/example/gradient_color.comp"),
    include_glsl!("resources/shaders/example/gradient.comp"),
];
const TRIANGLE_VERT: &[u32] = include_glsl!("resources/shaders/example/triangle.vert");
const TRIANGLE_FRAG: &[u32] = include_glsl!("resources/shaders/example/triangle.frag");
const MESH_VERT: &[u32] = include_glsl!("resources/shaders/example/mesh.vert");
const MESH_FRAG: &[u32] = include_glsl!("resources/shaders/example/mesh.frag");

#[repr(C)]
#[derive(Default)]
pub struct ComputePushConstants {
    pub data1: Vec4,
    pub data2: Vec4,
    pub data3: Vec4,
    pub data4: Vec4,
}

#[repr(C)]
pub struct GPUDrawPushConstants {
    pub world_matrix: Mat4,
    pub vertex_buffer_address: DeviceAddress,
}

pub struct ComputePipeline {
    pub name: String,
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

pub struct MeshPipeline {
    pub pipeline: Pipeline,
    pub pipeline_layout: PipelineLayout,
    pub shader_modules: Vec<ShaderModule>,
}

pub struct Pipelines {
    pub compute_pipelines: Vec<ComputePipeline>,
    pub active_compute_pipeline_index: usize,
    pub triangle_pipeline: GraphicsPipeline,
    pub mesh_pipeline: MeshPipeline,
}

impl MeshPipeline {
    pub fn new(logical_device: &Device) -> Self {
        let mut rendering_create_info = PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&[Format::R16G16B16A16_SFLOAT]) //DEFERRED ICIN BIRDEN FAZLA KOY!
            .depth_attachment_format(Format::D32_SFLOAT);

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
        let shader_module_create_info = ShaderModuleCreateInfo::default().code(MESH_VERT);
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

        let shader_module_create_info = ShaderModuleCreateInfo::default().code(MESH_FRAG);
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
            .topology(PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let rasterization_state_create_info = PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(PolygonMode::FILL)
            .cull_mode(ash::vk::CullModeFlags::BACK)
            .front_face(ash::vk::FrontFace::CLOCKWISE)
            .line_width(1.0);

        let multisample_state_create_info = PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(ash::vk::SampleCountFlags::TYPE_1);

        let depth_stencil_state_create_info = PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .depth_compare_op(CompareOp::GREATER_OR_EQUAL);

        let push_constant_ranges = [PushConstantRange::default()
            .offset(0)
            .size(size_of::<GPUDrawPushConstants>() as u32)
            .stage_flags(ShaderStageFlags::VERTEX)];

        let pipeline_layout_create_info =
            PipelineLayoutCreateInfo::default().push_constant_ranges(&push_constant_ranges);

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

impl GraphicsPipeline {
    pub fn new(logical_device: &Device) -> Self {
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
        let shader_module_create_info = ShaderModuleCreateInfo::default().code(TRIANGLE_VERT);
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

        let shader_module_create_info = ShaderModuleCreateInfo::default().code(TRIANGLE_FRAG);
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
            .topology(PrimitiveTopology::TRIANGLE_LIST)
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

        let pipeline_layout_create_info = PipelineLayoutCreateInfo::default();
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
        let compute_pipelines: Vec<ComputePipeline> = vec![
            ComputePipeline::new(
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
            ComputePipeline::new(
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
        let triangle_pipeline = GraphicsPipeline::new(logical_device);
        let mesh_pipeline = MeshPipeline::new(logical_device);
        Self {
            compute_pipelines,
            active_compute_pipeline_index: 0,
            triangle_pipeline,
            mesh_pipeline,
        }
    }

    pub fn get_current_compute_pipeline(&self) -> &ComputePipeline {
        &self.compute_pipelines[self.active_compute_pipeline_index]
    }

    pub fn get_current_compute_pipeline_mut(&mut self) -> &mut ComputePipeline {
        &mut self.compute_pipelines[self.active_compute_pipeline_index]
    }

    pub fn toggle_current_compute_pipeline(&mut self) {
        self.active_compute_pipeline_index =
            (self.active_compute_pipeline_index + 1) % self.compute_pipelines.len();
    }

    pub fn cleanup(&self, logical_device: &Device) {
        self.mesh_pipeline.cleanup(logical_device);
        self.triangle_pipeline.cleanup(logical_device);
        for effect in self.compute_pipelines.iter() {
            effect.cleanup(logical_device);
        }
    }
}
