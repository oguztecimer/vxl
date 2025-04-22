use crate::renderer::Renderer;
use ash::vk;
use ash::vk::{CommandBufferResetFlags, Fence, PipelineStageFlags, PresentInfoKHR, SubmitInfo};
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
            .create_window(WindowAttributes::default().with_title("vxl"))
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
                unsafe { self.renderer().device.logical.device_wait_idle() }
                    .expect("Could not wait for device idle");
                unsafe {
                    self.renderer().device.logical.reset_command_buffer(
                        self.renderer().command_buffer,
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
    fn window(&self) -> &Window { self.window.as_ref().unwrap() }
    
    fn draw_frame(&mut self) {
        if self.close_requested {
            return;
        }
        let fences = [self.renderer().sync.in_flight_fence];
        unsafe { self.renderer().device.logical.reset_fences(&fences) }
            .expect("Error in reset inflight fence");
        let mut image_index = None;
        let result = {
            unsafe {
                self.renderer()
                    .swapchain.loader
                    .acquire_next_image(
                        self.renderer().swapchain.handle,
                        u64::MAX,
                        self.renderer().sync.image_available_semaphore,
                        Fence::null(),
                    )
            }
        };
        match result {
            Ok((index, _)) => {
                image_index = Some(index);
            }
            Err(err) => {
                if !self.handle_error(err) {
                    return;
                }
            }
        }
        let image_index = image_index.unwrap();
        unsafe {
            self.renderer().device.logical.reset_command_buffer(
                self.renderer().command_buffer,
                CommandBufferResetFlags::default(),
            )
        }
        .expect("Could not reset command buffer");
        self.renderer().record_command_buffer(image_index as usize);
        let command_buffers = [self.renderer().command_buffer];
        let signal_semaphores = [self.renderer().sync.render_finished_semaphore];
        let wait_semaphores = [self.renderer().sync.image_available_semaphore];
        let queue = self.renderer().device.queues.graphics.1;
        let submit_info = SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT]);
        unsafe {
            self.renderer().device.logical.queue_submit(
                queue,
                &[submit_info],
                self.renderer().sync.in_flight_fence,
            )
        }
        .expect("Could not submit draw command buffer");
        let swap_chains = [self.renderer().swapchain.handle];
        let image_indices = [image_index];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swap_chains)
            .image_indices(&image_indices);
        let result = {
            unsafe { self.renderer().swapchain.loader.queue_present(queue, &present_info) }
        };
        match result {
            Ok(_) => (),
            Err(err) => {
                if !self.handle_error(err) {
                    return;
                }
            }
        }
        unsafe { self.renderer().device.logical.wait_for_fences(&fences, true, u64::MAX) }
        .expect("Error in wait for inflight fence");
        if !self.close_requested {
            self.window().request_redraw();
        }
    }

    fn handle_error(&mut self, err: vk::Result) -> bool {
        {
            if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                self.recreate_swap_chain()
            } else {
                panic!("{:?}", err);
            }
        }
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
