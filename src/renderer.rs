use std::ffi::CString;
use ash::{Device, Entry, Instance, vk};
use ash::khr::{surface, swapchain};
use ash::vk::*;
use vk_shader_macros::include_glsl;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

const VERT:&[u32] = include_glsl!("shaders/shader.vert");
const FRAG:&[u32] = include_glsl!("shaders/shader.frag");


pub struct Renderer{
    instance: Instance,
    physical_device: PhysicalDevice,
    surface: SurfaceKHR,
    surface_loader: surface::Instance,
    logical_device: Device,
    pub swap_chain: SwapchainKHR,
    pub queue_families: QueueFamilies,
    swap_chain_image_views: Vec<ImageView>,
    pub swap_chain_loader: swapchain::Device,
    swap_chain_extent: Extent2D,
    render_pass: RenderPass,
    layout: PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    frame_buffers: Vec<Framebuffer>,
    command_pool: CommandPool,
    pub command_buffer: CommandBuffer
}

pub struct SyncObjects{
    pub image_available_semaphore:Semaphore,
    pub render_finished_semaphore:Semaphore,
    pub in_flight_fence:Fence
}

pub struct QueueFamilies{
    pub graphics : (u32,Queue)
}

impl SyncObjects {
    pub fn new(logical_device: &Device) -> SyncObjects {
        let semaphore_create_info = SemaphoreCreateInfo::default();
        let fence_create_info = FenceCreateInfo::default();
        let image_available_semaphore = unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }
            .expect("Could not create semaphore");
        let render_finished_semaphore = unsafe { logical_device.create_semaphore(&semaphore_create_info, None) }
            .expect("Could not create semaphore");
        let in_flight_fence = unsafe { logical_device.create_fence(&fence_create_info, None) }
            .expect("Could not create fence");
        SyncObjects {
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence
        }
    }

    pub fn cleanup(&self,logical_device: &Device) {
        unsafe{logical_device.destroy_fence(self.in_flight_fence,None)};
        unsafe{logical_device.destroy_semaphore(self.image_available_semaphore,None)};
        unsafe{logical_device.destroy_semaphore(self.render_finished_semaphore,None)};
    }
}

impl Renderer{
    pub fn logical_device(&self) -> &Device {&self.logical_device}
    fn create_instance(window: &Window,entry: &Entry) -> Instance{
        let application_info = ApplicationInfo::default()
            .api_version(API_VERSION_1_3);
        let create_flags =
            if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };
        let display_handle = window.display_handle()
            .expect("Can't get raw display handle").as_raw();
        let mut extension_names = ash_window::enumerate_required_extensions(display_handle)
            .unwrap()
            .to_vec();
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
            // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
            extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }

        let create_info = InstanceCreateInfo::default()
            .application_info(&application_info)
            .flags(create_flags)
            .enabled_extension_names(&extension_names);
        unsafe{entry.create_instance(&create_info,None).expect("Instance creation err")}
    }
    fn create_physical_device_and_queue_family_index(
        instance: &Instance,
        surface_loader: &surface::Instance,
        surface: &SurfaceKHR
    ) -> (PhysicalDevice,u32){
        let physical_devices = unsafe{instance.enumerate_physical_devices()}
            .expect("Physical device error");
        if physical_devices.len() == 0{
            panic!("failed to find GPUs with Vulkan support!");
        }
        physical_devices.iter()
            .find_map(|&pd| {
                unsafe{instance.get_physical_device_queue_family_properties(pd)}
                    .iter()
                    .enumerate()
                    .find_map(|(index,&queue_family_properties)| {
                        let supports_graphic_and_surface =
                            queue_family_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && unsafe{surface_loader.get_physical_device_surface_support(
                                pd,
                                index as u32,
                                *surface,
                            )}
                                .unwrap();
                        if supports_graphic_and_surface {
                            Some((pd, index as u32))
                        } else {
                            None
                        }
                    })
            }).expect("Couldn't find suitable device")
    }


    fn create_logical_device(
        graphics_queue_family_index:u32,
        instance: &Instance,
        physical_device: PhysicalDevice
    ) -> Device{
        let device_queue_create_info = DeviceQueueCreateInfo::default()
            .queue_priorities(&[1.0])
            .queue_family_index(graphics_queue_family_index);
        let device_extension_names_raw = [swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
                ash::khr::portability_subset::NAME.as_ptr(),
        ];
        let device_queue_create_info_array = [device_queue_create_info];
        let create_device_info= DeviceCreateInfo::default()
            .queue_create_infos(&device_queue_create_info_array)
            .enabled_extension_names(&device_extension_names_raw);
        unsafe{instance.create_device
        (
            physical_device,
            &create_device_info,
            None
        )}.expect("Could not create logical device!")
    }

    fn create_surface(window:&Window, entry: &Entry, instance:&Instance) -> SurfaceKHR{
        let display_handle = window
            .display_handle()
            .expect("Can't get raw display handle").as_raw();
        let window_handle = window.window_handle()
            .expect("Can't get window handle")
            .as_raw();
        unsafe{ash_window::create_surface(
            &entry,
            &instance,
            display_handle,
            window_handle,
            None
        )}.expect("Could not create surface")
    }

    fn create_swap_chain(
        swap_chain_loader: &swapchain::Device,
        logical_device:&Device,
        surface_loader:&surface::Instance,
        physical_device: &PhysicalDevice,
        surface: &SurfaceKHR,
        queue_families: &QueueFamilies
    ) -> (
        SwapchainKHR,
        Format,
        Vec<ImageView>,
        Extent2D
    ){

        let surface_present_modes = unsafe{surface_loader
            .get_physical_device_surface_present_modes(*physical_device,*surface)}
            .expect("Could not get surface present modes.");
        let surface_capabilities = unsafe{surface_loader
            .get_physical_device_surface_capabilities(*physical_device,*surface)}
            .expect("Could not get surface capabilities");
        let surface_formats = unsafe{surface_loader
            .get_physical_device_surface_formats(*physical_device,*surface)}
            .expect("Could not get surface formats");
        let surface_present_mode = surface_present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == PresentModeKHR::MAILBOX)
            .unwrap_or(PresentModeKHR::FIFO);
        let min_image_count =
            (surface_capabilities.min_image_count+1).min(surface_capabilities.max_image_count);
        let image_format = surface_formats[0].format;
        let swap_chain_color_space = surface_formats[0].color_space;
        let swap_chain_extent = surface_capabilities.current_extent;
        let queue_family_indices_array = [queue_families.graphics.0];
        let swap_chain_create_info =
            SwapchainCreateInfoKHR::default()
                .surface(*surface)
                .min_image_count(min_image_count)
                .image_format(image_format)
                .image_color_space(swap_chain_color_space)
                .image_extent(swap_chain_extent)
                .image_array_layers(1)
                .image_usage(ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(SharingMode::EXCLUSIVE)
                .queue_family_indices(&queue_family_indices_array)
                .pre_transform(surface_capabilities.current_transform)
                .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(surface_present_mode)
            ;
        let swap_chain =
            unsafe{swap_chain_loader.create_swapchain(&swap_chain_create_info,None)}
                .expect("Could not create swap chain!");
        let images =
            unsafe{swap_chain_loader.get_swapchain_images(swap_chain)}
                .expect("Could not load swap chain images");
        let swap_chain_image_views:Vec<ImageView> =
            images
                .iter()
                .map(|&img|{
                    let subresource_range =
                        ImageSubresourceRange::default()
                            .aspect_mask(ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1);
                    let info =
                        ImageViewCreateInfo::default()
                            .subresource_range(subresource_range)
                            .image(img)
                            .view_type(ImageViewType::TYPE_2D)
                            .format(image_format);
                    unsafe{logical_device.create_image_view(&info,None)}
                        .unwrap()
                }).collect();
        (swap_chain,image_format,swap_chain_image_views,swap_chain_extent)
    }

    fn create_shader_module(logical_device: &Device,code:&[u32]) -> ShaderModule{
        let shader_module_create_info = ShaderModuleCreateInfo::default()
            .code(code);
        unsafe{logical_device.create_shader_module(&shader_module_create_info,None)}
            .expect("Could not create shader module")
    }

    fn create_frame_buffers(
        swap_chain_image_views:&Vec<ImageView>,
        render_pass: RenderPass,
        swap_chain_extent: Extent2D,
        logical_device: &Device

    )->Vec<Framebuffer>{
        swap_chain_image_views.iter().map(|&image_view|{
            let image_view_array = [image_view];
            let frame_buffer_create_info = FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&image_view_array)
                .width(swap_chain_extent.width)
                .height(swap_chain_extent.height)
                .layers(1);
            unsafe{logical_device.create_framebuffer(&frame_buffer_create_info,None)}
                .expect("Could not create frame buffer")
        }).collect()
    }

    fn create_command_pool(logical_device: &Device,graphics_queue_family_index:u32) -> CommandPool{
        let command_pool_create_info = CommandPoolCreateInfo::default()
            .queue_family_index(graphics_queue_family_index)
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        unsafe{logical_device.create_command_pool(&command_pool_create_info,None)}
            .expect("Could not create command pool")
    }

    pub fn record_command_buffer(&self, image_index:usize){
        let command_buffer_begin_info = CommandBufferBeginInfo::default();
        unsafe{self.logical_device.begin_command_buffer(self.command_buffer,&command_buffer_begin_info)}
            .expect("Could not begin recording the command buffer");

        let clear_values= [ClearValue{color: ClearColorValue{float32:[0.1,0.1,0.1,1.0]}}];
        let render_pass_begin_info = RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .clear_values(&clear_values)
            .framebuffer(self.frame_buffers[image_index])
            .render_area(Rect2D{offset:Offset2D{x:0,y:0},extent:self.swap_chain_extent});

        unsafe{self.logical_device.cmd_begin_render_pass(self.command_buffer,&render_pass_begin_info,SubpassContents::INLINE)};
        unsafe{self.logical_device.cmd_bind_pipeline(self.command_buffer,PipelineBindPoint::GRAPHICS,self.graphics_pipeline)};

        let viewport = Viewport::default()
            .x(0.0)
            .y(0.0)
            .min_depth(0.0)
            .max_depth(0.0)
            .width(self.swap_chain_extent.width as f32)
            .height(self.swap_chain_extent.height as f32);

        let scissor = Rect2D::default()
            .extent(self.swap_chain_extent)
            .offset(Offset2D{x:0,y:0});

        let viewports = [viewport];
        let scissors = [scissor];

        unsafe{self.logical_device.cmd_set_viewport(self.command_buffer,0,&viewports)}
        unsafe{self.logical_device.cmd_set_scissor(self.command_buffer,0,&scissors)}
        unsafe{self.logical_device.cmd_draw(self.command_buffer,3,1,0,0)}
        unsafe{self.logical_device.cmd_end_render_pass(self.command_buffer)}
        unsafe{self.logical_device.end_command_buffer(self.command_buffer)}
            .expect("Could not end recording command buffer");
    }

    //create_command_buffer

    pub fn new(window: &Window) -> Renderer{
        let entry = Entry::linked();
        let instance = Self::create_instance(window,&entry);
        let surface = Self::create_surface(window,&entry,&instance);
        let surface_loader = surface::Instance::new(&entry,&instance);
        let (physical_device,graphics_family_index) =
            Self::create_physical_device_and_queue_family_index(&instance,&surface_loader,&surface);
        let logical_device =
            Self::create_logical_device(graphics_family_index,&instance,physical_device);

        //swap_chain
        let queue = unsafe{logical_device.get_device_queue(graphics_family_index,0)};
        let queue_families = QueueFamilies{
            graphics : (graphics_family_index,queue)
        };
        let swap_chain_loader = swapchain::Device::new(&instance,&logical_device);

        let (swap_chain,image_format,swap_chain_image_views,swap_chain_extent) =
            Self::create_swap_chain(&swap_chain_loader,&logical_device,&surface_loader,&physical_device,&surface,&queue_families);

        let vert_module = Self::create_shader_module(&logical_device, VERT);
        let frag_module = Self::create_shader_module(&logical_device, FRAG);

        let name = CString::new("main").expect("Could not convert to CStr");
        let vertex_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::VERTEX)
            .module(vert_module).name(&name);
        let frag_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::FRAGMENT)
            .module(frag_module).name(&name);
        let stages = [vertex_info,frag_info];

        let dynamic_states = [DynamicState::VIEWPORT,DynamicState::SCISSOR];
        let dynamic_state_create_info =
            PipelineDynamicStateCreateInfo::default()
                .dynamic_states(&dynamic_states);

        let vertex_input_create_info =
            PipelineVertexInputStateCreateInfo::default();
        let input_assembly_state_create_info =
            PipelineInputAssemblyStateCreateInfo::default()
                .primitive_restart_enable(false)
                .topology(PrimitiveTopology::TRIANGLE_LIST);
        let pipeline_viewport_state_create_info =
            PipelineViewportStateCreateInfo::default()
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

        let pipeline_multisample_state_create_info =
            PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(SampleCountFlags::TYPE_1);

        let pipeline_stencil_state_create_info =
            PipelineDepthStencilStateCreateInfo::default();

        let pipeline_color_blend_attachment_state =
            PipelineColorBlendAttachmentState::default()
                .color_write_mask(ColorComponentFlags::RGBA)
                .blend_enable(false);

        let attachments = [pipeline_color_blend_attachment_state];
        let pipeline_color_blend_state_create_info =
            PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(&attachments);

        let pipeline_layout_create_info =
            PipelineLayoutCreateInfo::default();

        let layout = unsafe{logical_device
            .create_pipeline_layout(&pipeline_layout_create_info,None)}
            .expect("Could not create pipeline layout");

        //Render pass
        let color_attachment = AttachmentDescription::default()
            .samples(SampleCountFlags::TYPE_1)
            .format(image_format)
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

        let render_pass =
            unsafe{logical_device.create_render_pass(&render_pass_create_info,None)
                .expect("Could not create render pass")};

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
            .subpass(0)
            ;
        let graphics_pipeline_create_infos = [graphics_pipeline_create_info];
        let graphics_pipeline =
            unsafe{logical_device.create_graphics_pipelines(
                PipelineCache::null(),
                &graphics_pipeline_create_infos,
                None
            )
                .expect("Could not create graphics pipeline")}[0];

        unsafe{logical_device.destroy_shader_module(vert_module,None)};
        unsafe{logical_device.destroy_shader_module(frag_module,None)};


        let frame_buffers = Self::create_frame_buffers(
            &swap_chain_image_views,
            render_pass,
            swap_chain_extent,
            &logical_device
        );

        let command_pool =
            Self::create_command_pool(&logical_device,queue_families.graphics.0);

        let command_buffer_allocate_info =
            CommandBufferAllocateInfo::default()
                .command_pool(command_pool)
                .command_buffer_count(1)
                .level(CommandBufferLevel::PRIMARY);
        let command_buffer = unsafe{logical_device.allocate_command_buffers(&command_buffer_allocate_info)}
            .expect("Could not allocate command buffers")[0];

        Renderer{
            instance,
            physical_device,
            surface,
            surface_loader,
            logical_device,
            queue_families,
            swap_chain_loader,
            swap_chain,
            swap_chain_image_views,
            swap_chain_extent,
            layout,
            render_pass,
            graphics_pipeline,
            frame_buffers,
            command_pool,
            command_buffer
        }
    }

    pub fn recreate_swap_chain(&mut self){
        unsafe{self.logical_device().device_wait_idle()}
            .expect("Could not wait device idle");
        self.swap_chain_cleanup();
        let (swap_chain,_,swap_chain_image_views,swap_chain_extent) =
            Self::create_swap_chain(&self.swap_chain_loader,&self.logical_device,&self.surface_loader,&self.physical_device,&self.surface,&self.queue_families);
        self.frame_buffers = Self::create_frame_buffers(&swap_chain_image_views,self.render_pass,swap_chain_extent,&self.logical_device);
        self.swap_chain = swap_chain;
        self.swap_chain_image_views = swap_chain_image_views;
        self.swap_chain_extent = swap_chain_extent;
    }

    pub fn swap_chain_cleanup(&self){
        for fb in &self.frame_buffers {
            unsafe{self.logical_device.destroy_framebuffer(*fb, None)};
        }
        for view in &self.swap_chain_image_views{
            unsafe{self.logical_device.destroy_image_view(*view,None)};
        }
        unsafe{self.swap_chain_loader.destroy_swapchain(self.swap_chain,None)};
    }

    pub fn cleanup(&self,sync_objects: &SyncObjects){
        unsafe{self.logical_device.destroy_command_pool(self.command_pool,None)};
        unsafe{self.logical_device.destroy_pipeline(self.graphics_pipeline,None)};
        unsafe{self.logical_device.destroy_pipeline_layout(self.layout,None)};
        unsafe{self.logical_device.destroy_render_pass(self.render_pass,None)};
        sync_objects.cleanup(self.logical_device());
        self.swap_chain_cleanup();
        unsafe{self.surface_loader.destroy_surface(self.surface,None)};
        unsafe{self.logical_device.destroy_device(None)};
        unsafe{self.instance.destroy_instance(None)};
    }
}