use crate::renderer::Renderer;
use crate::renderer::images::transition_image_layout;
use ash::vk::{
    ClearColorValue, CommandBufferResetFlags, CommandBufferSubmitInfo, Fence, ImageAspectFlags,
    ImageLayout, ImageSubresourceRange, PipelineStageFlags2, PresentInfoKHR, SemaphoreSubmitInfo,
    SubmitInfo2,
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Default)]
pub struct App {
    pub window: Option<Window>,
    pub renderer: Option<Renderer>,
    pub close_requested: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title("vxl")
                    .with_inner_size(winit::dpi::LogicalSize::new(800.0, 800.0)),
            )
            .unwrap();
        self.renderer = Some(Renderer::new(&window));
        self.window = Some(window);
        self.window().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
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
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw_frame();
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
            &self.renderer().device.logical,
            command_buffer,
            self.renderer().swapchain.images[image_index],
            ImageLayout::UNDEFINED,
            ImageLayout::GENERAL,
        );

        let clear_color = ClearColorValue {
            float32: [1.0, 0.0, 0.0, 1.0],
        };
        let clear_range = ImageSubresourceRange::default()
            .aspect_mask(ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);
        let clear_ranges = [clear_range];
        unsafe {
            self.renderer().device.logical.cmd_clear_color_image(
                command_buffer,
                self.renderer().swapchain.images[image_index],
                ImageLayout::GENERAL,
                &clear_color,
                &clear_ranges,
            );
        }
        transition_image_layout(
            &self.renderer().device.logical,
            command_buffer,
            self.renderer().swapchain.images[image_index],
            ImageLayout::GENERAL,
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
            self.renderer().device.logical.queue_submit2(
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
        self.renderer_mut().commands.increment_frame();
    }

    fn recreate_swap_chain(&mut self) -> bool {
        let size = self.window().inner_size();
        if size.width == 0 || size.height == 0 {
            return false;
        }
        self.renderer_mut().recreate_swap_chain();
        true
    }
}
