use crate::imgui::{create_imgui_renderer, setup_imgui};
use crate::renderer::Renderer;
use crate::renderer::images::{copy_image_to_image, transition_image_layout};
use crate::renderer::pipelines::{ComputePushConstants, GPUDrawPushConstants};
use ash::vk::{
    AttachmentLoadOp, AttachmentStoreOp, ClearColorValue, ClearValue, CommandBuffer,
    CommandBufferResetFlags, CommandBufferSubmitInfo, CommandPool, Fence, ImageAspectFlags,
    ImageLayout, ImageSubresourceRange, ImageView, IndexType, Offset2D, PipelineBindPoint,
    PipelineStageFlags2, PresentInfoKHR, Rect2D, RenderingAttachmentInfo, RenderingInfo,
    SemaphoreSubmitInfo, ShaderStageFlags, SubmitInfo2, Viewport,
};
use glam::Mat4;
use imgui::Context;
use imgui_winit_support::WinitPlatform;
use winit::application::ApplicationHandler;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Default)]
pub struct App {
    pub window: Option<Window>,
    pub renderer: Option<Renderer>,
    pub imgui_context: Option<Context>,
    pub imgui_renderer: Option<imgui_rs_vulkan_renderer::Renderer>,
    pub imgui_platform: Option<WinitPlatform>,
    pub imgui_command_pool: Option<CommandPool>,
    pub close_requested: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("vxl")
                    .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0)),
            )
            .unwrap();

        let renderer = Renderer::new(&window);
        let (mut imgui_context, imgui_platform) = setup_imgui(&window);
        let (imgui_renderer, imgui_command_pool) = create_imgui_renderer(
            &renderer.instance.handle,
            &renderer.device,
            &mut imgui_context,
            None,
        );

        self.renderer = Some(renderer);
        self.window = Some(window);
        self.imgui_context = Some(imgui_context);
        self.imgui_platform = Some(imgui_platform);
        self.imgui_renderer = Some(imgui_renderer);
        self.imgui_command_pool = Some(imgui_command_pool);
        self.window().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let id = self.window().id();
        if let Some(imgui_platform) = self.imgui_platform.as_mut() {
            if let Some(imgui_context) = self.imgui_context.as_mut() {
                let generic_event: Event<WindowEvent> = Event::WindowEvent {
                    event: event.clone(),
                    window_id: id,
                };
                imgui_platform.handle_event(
                    imgui_context.io_mut(),
                    self.window.as_mut().unwrap(),
                    &generic_event,
                );
            }
        }

        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
                unsafe {
                    self.renderer()
                        .device
                        .logical
                        .device_wait_idle()
                        .expect("Could not wait for device idle");

                    self.renderer().device.logical.reset_command_buffer(
                        self.renderer().commands.get_current_frame().command_buffer,
                        CommandBufferResetFlags::default(),
                    )
                }
                .expect("Could not reset command buffer");
                unsafe {
                    self.renderer()
                        .device
                        .logical
                        .destroy_command_pool(self.imgui_command_pool.unwrap(), None)
                };
                self.imgui_platform = None;
                self.imgui_context = None;
                self.imgui_renderer = None;
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let window_size = self.window.as_ref().unwrap().inner_size();
                if let Some(imgui_context) = self.imgui_context.as_mut() {
                    imgui_context.io_mut().display_size =
                        [window_size.width as f32, window_size.height as f32];
                    self.draw_frame();
                }
            }
            WindowEvent::Resized(_) => {
                self.recreate_swap_chain();
            }

            _ => (),
        }
    }
}

impl App {
    fn renderer_mut(&mut self) -> &mut Renderer {
        self.renderer.as_mut().unwrap()
    }
    fn renderer(&self) -> &Renderer {
        self.renderer.as_ref().unwrap()
    }
    fn window(&self) -> &Window {
        self.window.as_ref().unwrap()
    }

    fn draw_frame(&mut self) {
        if self.close_requested {
            return;
        }
        let fences = [self.renderer().commands.get_current_frame().render_fence];
        unsafe {
            self.renderer()
                .device
                .logical
                .wait_for_fences(&fences, true, 1000000000)
                .expect("Could not wait for fences");
            self.renderer_mut().commands.increment_frame();
            let fences = [self.renderer().commands.get_current_frame().render_fence];

            self.renderer()
                .device
                .logical
                .reset_fences(&fences)
                .expect("Could not reset fences");
        }

        let image_index = {
            unsafe {
                self.renderer().swapchain.loader.acquire_next_image(
                    self.renderer().swapchain.handle,
                    1000000000,
                    self.renderer()
                        .commands
                        .get_current_frame()
                        .swapchain_semaphore,
                    Fence::null(),
                )
            }
        }
        .expect("Could not acquire next image")
        .0 as usize;

        let command_buffer = self.renderer().commands.get_current_frame().command_buffer;
        unsafe {
            self.renderer()
                .device
                .logical
                .reset_command_buffer(command_buffer, CommandBufferResetFlags::default())
        }
        .expect("Could not reset command buffer");
        self.renderer()
            .commands
            .begin_command_buffer(&self.renderer().device.logical);

        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.draw_image.image,
            ImageLayout::UNDEFINED,
            ImageLayout::GENERAL,
        );

        self.draw_background(command_buffer);
        self.draw_compute(command_buffer);

        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.draw_image.image,
            ImageLayout::GENERAL,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );
        self.draw_geometry(command_buffer);

        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.draw_image.image,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::TRANSFER_SRC_OPTIMAL,
        );
        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.images[image_index],
            ImageLayout::UNDEFINED,
            ImageLayout::TRANSFER_DST_OPTIMAL,
        );
        copy_image_to_image(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.draw_image.image,
            self.renderer().swapchain.images[image_index],
            self.renderer().swapchain.extent,
            self.renderer().swapchain.extent,
        );
        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.images[image_index],
            ImageLayout::TRANSFER_DST_OPTIMAL,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );
        self.draw_imgui(
            command_buffer,
            self.renderer().swapchain.image_views[image_index],
        );
        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.images[image_index],
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ImageLayout::PRESENT_SRC_KHR,
        );
        self.renderer()
            .commands
            .end_command_buffer(&self.renderer().device.logical);

        let command_buffer_submit_infos =
            [CommandBufferSubmitInfo::default().command_buffer(command_buffer)];
        let wait_semaphore_infos = [SemaphoreSubmitInfo::default()
            .semaphore(
                self.renderer()
                    .commands
                    .get_current_frame()
                    .swapchain_semaphore,
            )
            .stage_mask(PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)];
        let signal_semaphore_infos = [SemaphoreSubmitInfo::default()
            .semaphore(
                self.renderer()
                    .commands
                    .get_current_frame()
                    .render_semaphore,
            )
            .stage_mask(PipelineStageFlags2::ALL_GRAPHICS)];
        let submit_infos = [SubmitInfo2::default()
            .command_buffer_infos(&command_buffer_submit_infos)
            .wait_semaphore_infos(&wait_semaphore_infos)
            .signal_semaphore_infos(&signal_semaphore_infos)];

        unsafe {
            self.renderer().device.logical_sync2.queue_submit2(
                self.renderer().device.queues.graphics.1,
                &submit_infos,
                self.renderer().commands.get_current_frame().render_fence,
            )
        }
        .expect("Could not submit queue");
        let swapchains = [self.renderer().swapchain.handle];
        let image_indices = [image_index as u32];
        let wait_semaphores = [self
            .renderer()
            .commands
            .get_current_frame()
            .render_semaphore];
        let present_info = PresentInfoKHR::default()
            .swapchains(&swapchains)
            .wait_semaphores(&wait_semaphores)
            .image_indices(&image_indices);

        unsafe {
            self.renderer()
                .swapchain
                .loader
                .queue_present(self.renderer().device.queues.graphics.1, &present_info)
        }
        .expect("Could not present queue");
        if !self.close_requested {
            self.window().request_redraw();
        }
    }

    fn draw_background(&mut self, command_buffer: CommandBuffer) {
        let clear_color = ClearColorValue {
            float32: [0.0, 0.0, 0.0, 1.0],
        };
        let clear_range = ImageSubresourceRange::default()
            .aspect_mask(ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);
        let clear_ranges = [clear_range];
        unsafe {
            self.renderer().device.logical.cmd_clear_color_image(
                command_buffer,
                self.renderer().swapchain.draw_image.image,
                ImageLayout::GENERAL,
                &clear_color,
                &clear_ranges,
            );
        }
    }
    fn draw_compute(&mut self, command_buffer: CommandBuffer) {
        unsafe {
            self.renderer().device.logical.cmd_bind_pipeline(
                command_buffer,
                PipelineBindPoint::COMPUTE,
                self.renderer()
                    .pipelines
                    .get_current_compute_pipeline()
                    .pipeline,
            );
            let descriptor_sets = [self.renderer().descriptors.draw_image_descriptor_set];
            self.renderer().device.logical.cmd_bind_descriptor_sets(
                command_buffer,
                PipelineBindPoint::COMPUTE,
                self.renderer()
                    .pipelines
                    .get_current_compute_pipeline()
                    .pipeline_layout,
                0,
                &descriptor_sets,
                &[],
            );
            let push_constants = &self
                .renderer()
                .pipelines
                .get_current_compute_pipeline()
                .data;
            let push_constants_bytes: &[u8] = std::slice::from_raw_parts(
                push_constants as *const ComputePushConstants as *const u8,
                size_of::<ComputePushConstants>(),
            );

            self.renderer().device.logical.cmd_push_constants(
                command_buffer,
                self.renderer()
                    .pipelines
                    .get_current_compute_pipeline()
                    .pipeline_layout,
                ShaderStageFlags::COMPUTE,
                0,
                push_constants_bytes,
            );
            let extent = self.renderer().swapchain.extent;
            self.renderer().device.logical.cmd_dispatch(
                command_buffer,
                extent.width / 16,
                extent.height / 16,
                1,
            );
        }
    }

    fn draw_geometry(&mut self, command_buffer: CommandBuffer) {
        let attachment_info = self.create_rendering_attachment_info(
            self.renderer().swapchain.draw_image.image_view,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            None,
        );
        let color_attachments = [attachment_info];
        let rendering_info = RenderingInfo::default()
            .color_attachments(&color_attachments)
            .layer_count(1)
            .render_area(Rect2D {
                offset: Offset2D::default(),
                extent: self.renderer().swapchain.extent,
            });
        let viewport = Viewport::default()
            .width(self.renderer().swapchain.extent.width as f32)
            .height(self.renderer().swapchain.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        let scissor = Rect2D {
            offset: Offset2D::default(),
            extent: self.renderer().swapchain.extent,
        };

        unsafe {
            let logical_device = &self.renderer().device.logical;
            let logical_device_dyn = &self.renderer().device.logical_dynamic_rendering;
            logical_device_dyn.cmd_begin_rendering(command_buffer, &rendering_info);
            logical_device.cmd_bind_pipeline(
                command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.renderer().pipelines.triangle_pipeline.pipeline,
            );
            logical_device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            logical_device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            //logical_device.cmd_draw(command_buffer, 3, 1, 0, 0);

            logical_device.cmd_bind_pipeline(
                command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.renderer().pipelines.mesh_pipeline.pipeline,
            );

            let world_matrix = Mat4::IDENTITY;
            let push_constants = &GPUDrawPushConstants {
                world_matrix,
                vertex_buffer_address: self.renderer().test_gpu_mesh_buffers.vertex_buffer_address,
            };
            let push_constants_bytes: &[u8] = std::slice::from_raw_parts(
                push_constants as *const GPUDrawPushConstants as *const u8,
                size_of::<GPUDrawPushConstants>(),
            );

            logical_device.cmd_push_constants(
                command_buffer,
                self.renderer().pipelines.mesh_pipeline.pipeline_layout,
                ShaderStageFlags::VERTEX,
                0,
                push_constants_bytes,
            );
            logical_device.cmd_bind_index_buffer(
                command_buffer,
                self.renderer().test_gpu_mesh_buffers.index_buffer.buffer,
                0,
                IndexType::UINT32,
            );
            logical_device.cmd_draw_indexed(command_buffer, 6, 1, 0, 0, 0);
            logical_device_dyn.cmd_end_rendering(command_buffer);
        }
    }

    fn draw_imgui(&mut self, command_buffer: CommandBuffer, target_image_view: ImageView) {
        let color_attachment = self.create_rendering_attachment_info(
            target_image_view,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            None,
        );
        let color_attachments = [color_attachment];
        let rendering_info = RenderingInfo::default()
            .color_attachments(&color_attachments)
            .layer_count(1)
            .render_area(Rect2D {
                offset: Offset2D::default(),
                extent: self.renderer().swapchain.extent,
            });
        let active_effect_name = self
            .renderer()
            .pipelines
            .get_current_compute_pipeline()
            .name
            .clone();
        unsafe {
            self.renderer()
                .device
                .logical_dynamic_rendering
                .cmd_begin_rendering(command_buffer, &rendering_info);
            let frame_number = self.renderer().commands.frame_number;
            let pipelines = &mut self.renderer.as_mut().unwrap().pipelines;
            let imgui_context_mut = self.imgui_context.as_mut().unwrap();
            let imgui_renderer_mut = self.imgui_renderer.as_mut().unwrap();
            let imgui_platform_mut = self.imgui_platform.as_mut().unwrap();

            let window = self.window.as_ref().unwrap();
            imgui_platform_mut
                .prepare_frame(imgui_context_mut.io_mut(), window)
                .expect("Failed to prepare frame");
            let ui = imgui_context_mut.frame();

            // Get a single mutable reference to the current effect's data
            let effect_data = &mut pipelines.get_current_compute_pipeline_mut().data;

            // Mutable references to data1 and data2 fields
            let data1 = &mut effect_data.data1;
            let data2 = &mut effect_data.data2;
            let mut toggle_shader = false;

            ui.window("Debug")
                .size([400.0, 200.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    {
                        ui.text(format!("Frame: {}", frame_number));
                    }

                    ui.slider("x", 0f32, 1f32, &mut data1.x);
                    ui.slider("y", 0f32, 1f32, &mut data1.y);
                    ui.slider("z", 0f32, 1f32, &mut data1.z);

                    ui.slider("x2", 0f32, 1f32, &mut data2.x);
                    ui.slider("y2", 0f32, 1f32, &mut data2.y);
                    ui.slider("z2", 0f32, 1f32, &mut data2.z);

                    if ui.button(format!("Toggle Shader - {}", active_effect_name)) {
                        toggle_shader = true;
                    }
                });
            if toggle_shader {
                pipelines.toggle_current_compute_pipeline();
            }
            imgui_platform_mut.prepare_render(ui, window);
            let draw_data = imgui_context_mut.render();
            imgui_renderer_mut
                .cmd_draw(command_buffer, draw_data)
                .expect("Could not draw imgui");
            self.renderer()
                .device
                .logical_dynamic_rendering
                .cmd_end_rendering(command_buffer);
        }
    }

    fn recreate_swap_chain(&mut self) -> bool {
        let size = self.window().inner_size();
        if size.width == 0 || size.height == 0 {
            return false;
        }
        self.renderer_mut().recreate_swap_chain();
        self.renderer().descriptors.update(
            &self.renderer().device.logical,
            self.renderer().swapchain.draw_image.image_view,
        );
        true
    }

    fn create_rendering_attachment_info(
        &self,
        view: ImageView,
        layout: ImageLayout,
        clear: Option<ClearValue>,
    ) -> RenderingAttachmentInfo {
        let mut info = RenderingAttachmentInfo::default()
            .image_view(view)
            .image_layout(layout)
            .load_op(if clear.is_some() {
                AttachmentLoadOp::CLEAR
            } else {
                AttachmentLoadOp::LOAD
            })
            .store_op(AttachmentStoreOp::STORE);
        if let Some(clear) = clear {
            info.clear_value = clear;
        }
        info
    }
}
