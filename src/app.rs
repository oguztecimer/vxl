use ash::Device;
use ash::vk::{CommandBufferResetFlags, Fence, PipelineStageFlags, PresentInfoKHR, SubmitInfo};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop};
use winit::window::{Window, WindowAttributes, WindowId};
use crate::renderer::{Renderer, SyncObjects};

#[derive(Default)]
pub struct App{
    pub window: Option<Window>,
    pub renderer: Option<Renderer>,
    pub sync_objects: Option<SyncObjects>,
    pub close_requested: bool
}

impl ApplicationHandler for App{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window
        (
            WindowAttributes::default()
                .with_title("vxl")
        ).unwrap();
        self.renderer = Some(Renderer::new(&window));
        self.window = Some(window);
        self.window.as_ref().unwrap().request_redraw();
        self.sync_objects = Some(SyncObjects::new(self.renderer.as_ref().unwrap().logical_device()));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
                unsafe{self.renderer().logical_device().device_wait_idle()}
                    .expect("Could not wait for device idle");
                unsafe{self.renderer().logical_device().reset_command_buffer(self.renderer().command_buffer,CommandBufferResetFlags::default())}
                     .expect("Could not reset command buffer");
                self.renderer().cleanup(self.sync_objects.as_ref().unwrap());
                event_loop.exit();
                
            },
            WindowEvent::RedrawRequested => {
                if !self.close_requested {
                    self.draw_frame(self.renderer().logical_device());
                }
            },
            WindowEvent::Resized(_)=>{
                self.window.as_ref().unwrap().request_redraw();
            }
            _=>()
        }
    }
}

impl App{
    fn renderer(&self) -> &Renderer{self.renderer.as_ref().unwrap()}
    fn draw_frame(&self,logical_device:&Device){
        let sync_objects = self.sync_objects.as_ref().unwrap();
        let fences = [sync_objects.in_flight_fence];
        let (image_index,_) = unsafe{self.renderer().swap_chain().loader.acquire_next_image(
            self.renderer().swap_chain().swap_chain,
            u64::MAX,
            sync_objects.image_available_semaphore,
            Fence::null()
        )}
            .expect("Could not acquire next image");
        unsafe{logical_device.reset_command_buffer(self.renderer().command_buffer,CommandBufferResetFlags::default())}
            .expect("Could not reset command buffer");
        self.renderer().record_command_buffer(image_index as usize);
        let command_buffers = [self.renderer().command_buffer];
        let signal_semaphores = [sync_objects.render_finished_semaphore];
        let wait_semaphores = [sync_objects.image_available_semaphore];

        let submit_info = SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT]);

        let queue = unsafe{logical_device.get_device_queue(self.renderer().swap_chain()
                                                               .queue_family_indices.graphics,0)};
        unsafe{logical_device.queue_submit(
            queue,
            &[submit_info],
            sync_objects.in_flight_fence)}
            .expect("Could not submit draw command buffer");
        let swap_chains = [self.renderer().swap_chain().swap_chain];
        let image_indices = [image_index];
        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swap_chains)
            .image_indices(&image_indices);
        unsafe {
            self.renderer().swap_chain().loader.queue_present(queue, &present_info)
                .expect("Could not present queue");
        }
        unsafe{logical_device.wait_for_fences(&fences,true,u64::MAX)}
            .expect("Error in wait for inflight fence");
        unsafe{logical_device.reset_fences(&fences)}
            .expect("Error in reset inflight fence");
        if !self.close_requested{
            self.window.as_ref().unwrap().request_redraw();
        }
    }
}