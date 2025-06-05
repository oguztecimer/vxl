use crate::Renderer;
use crate::images::{copy_image_to_image, transition_image_layout};
use crate::imgui::{create_imgui_renderer, setup_imgui};
use crate::pipelines::{MapEditorPushConstants, PushConstants};
use ash::vk::{
    AccessFlags2, AttachmentLoadOp, AttachmentStoreOp, ClearColorValue, ClearValue, CommandBuffer,
    CommandBufferResetFlags, CommandBufferSubmitInfo, CommandPool, DependencyInfo, Fence,
    ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageView, MemoryBarrier2, Offset2D,
    PipelineBindPoint, PipelineStageFlags2, PresentInfoKHR, Rect2D, RenderingAttachmentInfo,
    RenderingInfo, SemaphoreSubmitInfo, ShaderStageFlags, SubmitInfo2, Viewport,
};
use glam::{IVec2, Vec2, vec2};
use imgui::Context;
use imgui_winit_support::WinitPlatform;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalPosition;
use winit::event::{ElementState, Event, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
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
    pub last_frame: Option<Instant>,
    pub mouse_pressed: bool,
    pub mouse_pos: IVec2,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("vxl")
                    .with_inner_size(winit::dpi::LogicalSize::new(1200.0, 900.0)),
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
        self.last_frame = Some(Instant::now());
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
                let frame_start_time = Instant::now();
                if let Some(imgui_context) = self.imgui_context.as_mut() {
                    imgui_context.io_mut().display_size =
                        [window_size.width as f32, window_size.height as f32];
                    self.draw_frame();
                }
                let time_step = 0.02;
                let frame_duration = Instant::now()
                    .duration_since(frame_start_time)
                    .as_secs_f64();
                let diff = time_step - frame_duration;
                if diff > 0.0 {
                    std::thread::sleep(std::time::Duration::from_secs_f64(diff));
                }
            }
            WindowEvent::Resized(_) => {
                self.recreate_swap_chain();
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let mut mouse_pressed = false;
                if button == MouseButton::Left
                    && state == ElementState::Pressed
                    && !self.imgui_context.as_ref().unwrap().io().want_capture_mouse
                {
                    mouse_pressed = true;
                }
                self.mouse_pressed = mouse_pressed;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let pos: LogicalPosition<i32> = position.to_logical(5.0);
                self.mouse_pos = IVec2::new(pos.x, pos.y);
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
        let steps = 20;
        let current_frame = Instant::now();

        let scale = 5.0;
        let source = scale * 512.0;
        let uv_min = vec2(
            1.0 - (self.renderer().swapchain.extent.width as f32 / source),
            1.0 - (self.renderer().swapchain.extent.height as f32 / source),
        ) / 2.0;
        let uv_max = vec2(1.0, 1.0) - uv_min;
        self.last_frame = Some(current_frame);
        self.renderer_mut()
            .pipelines
            .simulation_pipeline
            .data
            .update(uv_min, uv_max);
        self.renderer_mut()
            .pipelines
            .render_pipeline
            .data
            .update(uv_min, uv_max);
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
            self.renderer().swapchain.simulation_image0.image,
            ImageLayout::UNDEFINED,
            ImageLayout::GENERAL,
        );
        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.simulation_image1.image,
            ImageLayout::UNDEFINED,
            ImageLayout::GENERAL,
        );
        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.simulation_image2.image,
            ImageLayout::UNDEFINED,
            ImageLayout::GENERAL,
        );

        if self.mouse_pressed {
            self.run_map_editor(command_buffer, uv_min);
        }
        for _ in 0..steps {
            self.clear_flags(command_buffer);
            self.run_simulation(command_buffer);
        }

        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.simulation_image1.image,
            ImageLayout::GENERAL,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.simulation_image2.image,
            ImageLayout::GENERAL,
            ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );

        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.render_image.image,
            ImageLayout::UNDEFINED,
            ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );

        self.draw_screen(command_buffer);

        transition_image_layout(
            &self.renderer().device,
            command_buffer,
            self.renderer().swapchain.render_image.image,
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
            self.renderer().swapchain.render_image.image,
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

    fn clear_flags(&mut self, command_buffer: CommandBuffer) {
        let clear_color = ClearColorValue {
            uint32: [0, 0, 0, 0],
        };
        let clear_range = ImageSubresourceRange::default()
            .aspect_mask(ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);
        let clear_ranges = [clear_range];
        unsafe {
            self.renderer().device.logical.cmd_clear_color_image(
                command_buffer,
                self.renderer().swapchain.simulation_image0.image,
                ImageLayout::GENERAL,
                &clear_color,
                &clear_ranges,
            );
        }
    }

    fn run_map_editor(&mut self, command_buffer: CommandBuffer, uv_min: Vec2) {
        unsafe {
            self.renderer().device.logical.cmd_bind_pipeline(
                command_buffer,
                PipelineBindPoint::COMPUTE,
                self.renderer().pipelines.map_editor_pipeline.pipeline,
            );
            let descriptor_sets = [self.renderer().descriptors.map_editor_descriptor_set];
            self.renderer().device.logical.cmd_bind_descriptor_sets(
                command_buffer,
                PipelineBindPoint::COMPUTE,
                self.renderer()
                    .pipelines
                    .map_editor_pipeline
                    .pipeline_layout,
                0,
                &descriptor_sets,
                &[],
            );

            let mut push_constants = self.renderer().pipelines.map_editor_pipeline.data;
            push_constants.update(
                self.mouse_pos + IVec2::new((uv_min.x * 512.0) as i32, (uv_min.y * 512.0) as i32),
                1,
            );
            let push_constants_bytes: &[u8] = std::slice::from_raw_parts(
                &push_constants as *const MapEditorPushConstants as *const u8,
                size_of::<MapEditorPushConstants>(),
            );
            self.renderer().device.logical.cmd_push_constants(
                command_buffer,
                self.renderer()
                    .pipelines
                    .map_editor_pipeline
                    .pipeline_layout,
                ShaderStageFlags::COMPUTE,
                0,
                push_constants_bytes,
            );
            self.renderer()
                .device
                .logical
                .cmd_dispatch(command_buffer, 1, 1, 1);
        }
    }

    fn run_simulation(&mut self, command_buffer: CommandBuffer) {
        unsafe {
            self.renderer().device.logical.cmd_bind_pipeline(
                command_buffer,
                PipelineBindPoint::COMPUTE,
                self.renderer().pipelines.simulation_pipeline.pipeline,
            );
            let descriptor_sets = [self.renderer().descriptors.simulation_descriptor_set];
            self.renderer().device.logical.cmd_bind_descriptor_sets(
                command_buffer,
                PipelineBindPoint::COMPUTE,
                self.renderer()
                    .pipelines
                    .simulation_pipeline
                    .pipeline_layout,
                0,
                &descriptor_sets,
                &[],
            );
            for batch in 0..9 {
                // can be increased to increase max movement per frame
                if batch != 0 {
                    let barrier = MemoryBarrier2::default()
                        .src_access_mask(AccessFlags2::SHADER_STORAGE_WRITE)
                        .dst_access_mask(AccessFlags2::SHADER_STORAGE_READ)
                        .src_stage_mask(PipelineStageFlags2::COMPUTE_SHADER)
                        .dst_stage_mask(PipelineStageFlags2::COMPUTE_SHADER);
                    let barriers = [barrier];
                    let dependency_info = DependencyInfo::default().memory_barriers(&barriers);
                    self.renderer()
                        .device
                        .logical_sync2
                        .cmd_pipeline_barrier2(command_buffer, &dependency_info);
                }
                let mut push_constants = self.renderer().pipelines.simulation_pipeline.data;
                push_constants.batch_offset = match batch {
                    1 => IVec2::new(1, 0),
                    2 => IVec2::new(2, 0),
                    3 => IVec2::new(0, 1),
                    4 => IVec2::new(1, 1),
                    5 => IVec2::new(2, 1),
                    6 => IVec2::new(0, 2),
                    7 => IVec2::new(1, 2),
                    8 => IVec2::new(2, 2),
                    _ => IVec2::new(0, 0),
                };
                let push_constants_bytes: &[u8] = std::slice::from_raw_parts(
                    &push_constants as *const PushConstants as *const u8,
                    size_of::<PushConstants>(),
                );
                self.renderer().device.logical.cmd_push_constants(
                    command_buffer,
                    self.renderer()
                        .pipelines
                        .simulation_pipeline
                        .pipeline_layout,
                    ShaderStageFlags::COMPUTE,
                    0,
                    push_constants_bytes,
                );
                self.renderer().device.logical.cmd_dispatch(
                    command_buffer,
                    22, // = 512 / (16)
                    22,
                    1,
                );
            }
        }
    }

    fn draw_screen(&mut self, command_buffer: CommandBuffer) {
        let attachment_info = self.create_rendering_attachment_info(
            self.renderer().swapchain.render_image.image_view,
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
            .height(self.renderer().swapchain.extent.height as f32);
        let scissor = Rect2D {
            offset: Offset2D::default(),
            extent: self.renderer().swapchain.extent,
        };

        let push_constants = &self.renderer().pipelines.render_pipeline.data;

        unsafe {
            let push_constants_bytes: &[u8] = std::slice::from_raw_parts(
                push_constants as *const PushConstants as *const u8,
                size_of::<PushConstants>(),
            );
            let logical_device = &self.renderer().device.logical;
            let logical_device_dyn = &self.renderer().device.logical_dynamic_rendering;
            logical_device_dyn.cmd_begin_rendering(command_buffer, &rendering_info);
            logical_device.cmd_bind_pipeline(
                command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.renderer().pipelines.render_pipeline.pipeline,
            );
            let descriptor_sets = [self.renderer().descriptors.render_descriptor_set];
            self.renderer().device.logical.cmd_bind_descriptor_sets(
                command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.renderer().pipelines.render_pipeline.pipeline_layout,
                0,
                &descriptor_sets,
                &[],
            );
            self.renderer().device.logical.cmd_push_constants(
                command_buffer,
                self.renderer().pipelines.render_pipeline.pipeline_layout,
                ShaderStageFlags::FRAGMENT,
                0,
                push_constants_bytes,
            );
            logical_device.cmd_set_viewport(command_buffer, 0, &[viewport]);
            logical_device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            logical_device.cmd_draw(command_buffer, 4, 1, 0, 0);

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
        unsafe {
            self.renderer()
                .device
                .logical_dynamic_rendering
                .cmd_begin_rendering(command_buffer, &rendering_info);
            let imgui_context_mut = self.imgui_context.as_mut().unwrap();
            let imgui_renderer_mut = self.imgui_renderer.as_mut().unwrap();
            let imgui_platform_mut = self.imgui_platform.as_mut().unwrap();
            let window = self.window.as_ref().unwrap();
            imgui_platform_mut
                .prepare_frame(imgui_context_mut.io_mut(), window)
                .expect("Failed to prepare frame");
            let ui = imgui_context_mut.frame();
            ui.show_demo_window(&mut true);
            ui.window("Debug")
                .size([400.0, 200.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    // {
                    //     ui.text(format!("Frame: {}", frame_number));
                    // }

                    ui.color_button_config("deneme", [1.0, 0.0, 0.0, 1.0])
                        .border(true)
                        .tooltip(false)
                        .build();
                    if ui.color_button("Debug Color", [1.0, 0.0, 0.0, 1.0]) {
                        dbg!("Debug Color");
                    }
                    ui.same_line_with_spacing(0.0, 10.0);
                    if ui.color_button("Debug Color", [1.0, 1.0, 0.0, 1.0]) {
                        dbg!("Debug Color");
                    }
                    //ui.show_demo_window(&mut true);
                });
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
        self.renderer()
            .descriptors
            .update(&self.renderer().device.logical, &self.renderer().swapchain);
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

    pub fn run() {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App::default();
        event_loop
            .run_app(&mut app)
            .expect("Could not run event loop");
    }
}
