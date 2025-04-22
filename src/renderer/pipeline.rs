use std::ffi::CString;
use ash::vk::{AccessFlags, AttachmentDescription, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, ColorComponentFlags, CullModeFlags, DynamicState, FrontFace, GraphicsPipelineCreateInfo, ImageLayout, PipelineBindPoint, PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo, PipelineDepthStencilStateCreateInfo, PipelineDynamicStateCreateInfo, PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo, PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo, PipelineShaderStageCreateInfo, PipelineStageFlags, PipelineVertexInputStateCreateInfo, PipelineViewportStateCreateInfo, PolygonMode, PrimitiveTopology, RenderPass, RenderPassCreateInfo, SampleCountFlags, ShaderModule, ShaderModuleCreateInfo, ShaderStageFlags, SubpassDependency, SubpassDescription, SUBPASS_EXTERNAL};
use vk_shader_macros::include_glsl;
use crate::renderer::device::Device;
use crate::renderer::swapchain::Swapchain;
use crate::renderer::vertex::Vertex;

const VERT: &[u32] = include_glsl!("resources/shaders/shader.vert");
const FRAG: &[u32] = include_glsl!("resources/shaders/shader.frag");



pub struct Pipeline{
    pub handle: ash::vk::Pipeline,
    pub layout: PipelineLayout,
    pub render_pass: RenderPass
}

impl Pipeline{
    pub fn new(
        device: &Device,
        swapchain: &Swapchain,
    ) -> Self{
        let vert_module = Self::create_shader_module(device, VERT);
        let frag_module = Self::create_shader_module(device, FRAG);

        let name = CString::new("main").expect("Could not convert to CStr");
        let vertex_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::VERTEX)
            .module(vert_module)
            .name(&name);
        let frag_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .name(&name);
        let stages = [vertex_info, frag_info];

        let dynamic_states = [DynamicState::VIEWPORT, DynamicState::SCISSOR];
        let dynamic_state_create_info =
            PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        let binding_descriptions = Vertex::get_binding_descriptions();
        let binding_attributes = Vertex::get_attribute_descriptions();
        let vertex_input_create_info = PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&binding_attributes);
        let input_assembly_state_create_info = PipelineInputAssemblyStateCreateInfo::default()
            .primitive_restart_enable(false)
            .topology(PrimitiveTopology::TRIANGLE_LIST);
        let pipeline_viewport_state_create_info = PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let pipeline_rasterization_state_create_info =
            PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(CullModeFlags::BACK)
                .front_face(FrontFace::CLOCKWISE)
                .depth_bias_enable(false);

        let pipeline_multisample_state_create_info = PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(SampleCountFlags::TYPE_1);

        let pipeline_stencil_state_create_info = PipelineDepthStencilStateCreateInfo::default();

        let pipeline_color_blend_attachment_state = PipelineColorBlendAttachmentState::default()
            .color_write_mask(ColorComponentFlags::RGBA)
            .blend_enable(false);

        let attachments = [pipeline_color_blend_attachment_state];
        let pipeline_color_blend_state_create_info = PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&attachments);

        let pipeline_layout_create_info = PipelineLayoutCreateInfo::default();

        let layout =
            unsafe { device.logical.create_pipeline_layout(&pipeline_layout_create_info, None) }
                .expect("Could not create pipeline layout");

        //Render pass
        let color_attachment = AttachmentDescription::default()
            .samples(SampleCountFlags::TYPE_1)
            .format(swapchain.image_format)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = AttachmentReference::default()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachments = [color_attachment];
        let color_attachment_refs = [color_attachment_ref];

        let sub_pass_description = SubpassDescription::default()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs);
        let sub_pass_descriptions = [sub_pass_description];

        let dependencies = [SubpassDependency::default()
            .src_subpass(SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(AccessFlags::NONE)
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let render_pass_create_info = RenderPassCreateInfo::default()
            .attachments(&color_attachments)
            .subpasses(&sub_pass_descriptions)
            .dependencies(&dependencies);

        let render_pass = unsafe {
            device.logical
                .create_render_pass(&render_pass_create_info, None)
                .expect("Could not create render pass")
        };

        //Pipeline

        let graphics_pipeline_create_info = GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input_create_info)
            .input_assembly_state(&input_assembly_state_create_info)
            .viewport_state(&pipeline_viewport_state_create_info)
            .rasterization_state(&pipeline_rasterization_state_create_info)
            .multisample_state(&pipeline_multisample_state_create_info)
            .depth_stencil_state(&pipeline_stencil_state_create_info)
            .color_blend_state(&pipeline_color_blend_state_create_info)
            .dynamic_state(&dynamic_state_create_info)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0);
        let graphics_pipeline_create_infos = [graphics_pipeline_create_info];
        let handle = unsafe {
            device.logical
                .create_graphics_pipelines(
                    PipelineCache::null(),
                    &graphics_pipeline_create_infos,
                    None,
                )
                .expect("Could not create graphics pipeline")
        }[0];

        unsafe {
            device.logical.destroy_shader_module(vert_module, None);
            device.logical.destroy_shader_module(frag_module, None)
        };

        Self{
            handle,
            layout,
            render_pass
        }
    }
    fn create_shader_module(device: &Device, code: &[u32]) -> ShaderModule {
        let shader_module_create_info = ShaderModuleCreateInfo::default().code(code);
        unsafe { device.logical.create_shader_module(&shader_module_create_info, None) }
            .expect("Could not create shader module")
    }
    
    pub fn cleanup(&self, logical_device: &ash::Device) {
        unsafe {
            logical_device.destroy_pipeline(self.handle, None);
            logical_device.destroy_pipeline_layout(self.layout, None);
            logical_device.destroy_render_pass(self.render_pass, None);
        }
    }
}