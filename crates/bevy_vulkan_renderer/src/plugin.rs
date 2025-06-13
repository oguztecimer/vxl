use crate::BevyVulkanRenderer;
use crate::images::{copy_image_to_image, transition_image_layout};
use crate::pipelines::{MapEditorPushConstants, PushConstants};
use ash::vk::{
    AccessFlags2, AttachmentLoadOp, AttachmentStoreOp, ClearColorValue, ClearValue, CommandBuffer,
    CommandBufferResetFlags, CommandBufferSubmitInfo, DependencyInfo, Fence, ImageAspectFlags,
    ImageLayout, ImageSubresourceRange, ImageView, MemoryBarrier2, Offset2D, PipelineBindPoint,
    PipelineStageFlags2, PresentInfoKHR, Rect2D, RenderingAttachmentInfo, RenderingInfo,
    SemaphoreSubmitInfo, ShaderStageFlags, SubmitInfo2, Viewport,
};
use bevy::ecs::entity::unique_slice::Windows;
use bevy::prelude::{App, Commands, Entity, EventReader, IntoScheduleConfigs, NonSend, Plugin, PostUpdate, Query, Res, ResMut, Startup, Window};
use bevy::window::WindowResized;
use bevy::winit::WinitWindows;
use glam::{IVec2, Vec2, vec2};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct BevyVulkanPlugin;

impl Plugin for BevyVulkanPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_vulkan_renderer)
            .add_systems(PostUpdate, draw_frame)
            .add_systems(PostUpdate, handle_window_resize.before(draw_frame))
        ;
    }
}

fn setup_vulkan_renderer(
    mut commands: Commands,
    windows: Query<(Entity, &Window)>,
    winit_windows: NonSend<WinitWindows>,
) {
    let (window_entity, _window) = windows.single().expect("None");

    let winit_window = winit_windows
        .get_window(window_entity)
        .expect("Failed to get winit window");

    let raw_display_handle = winit_window
        .display_handle()
        .expect("Failed to get raw window handle")
        .as_raw();

    let raw_window_handle = winit_window
        .window_handle()
        .expect("Failed to get raw window handle")
        .as_raw();

    let renderer = BevyVulkanRenderer::new(raw_display_handle, raw_window_handle);
    commands.insert_resource(renderer);
}

fn draw_frame(mut renderer: ResMut<BevyVulkanRenderer>) {
    let steps = 20;

    let scale = 5.0;
    let source = scale * 512.0;
    let uv_min = vec2(
        1.0 - (renderer.swapchain.extent.width as f32 / source),
        1.0 - (renderer.swapchain.extent.height as f32 / source),
    ) / 2.0;
    let uv_max = vec2(1.0, 1.0) - uv_min;
    renderer
        .pipelines
        .simulation_pipeline
        .data
        .update(uv_min, uv_max);
    renderer
        .pipelines
        .render_pipeline
        .data
        .update(uv_min, uv_max);
    let fences = [renderer.commands.get_current_frame().render_fence];
    unsafe {
        renderer
            .device
            .logical
            .wait_for_fences(&fences, true, 1000000000)
            .expect("Could not wait for fences");
        renderer.commands.increment_frame();
        let fences = [renderer.commands.get_current_frame().render_fence];

        renderer
            .device
            .logical
            .reset_fences(&fences)
            .expect("Could not reset fences");
    }

    let image_index = {
        unsafe {
            renderer.swapchain.loader.acquire_next_image(
                renderer.swapchain.handle,
                1000000000,
                renderer.commands.get_current_frame().swapchain_semaphore,
                Fence::null(),
            )
        }
    }
    .expect("Could not acquire next image")
    .0 as usize;
    let command_buffer = renderer.commands.get_current_frame().command_buffer;
    unsafe {
        renderer
            .device
            .logical
            .reset_command_buffer(command_buffer, CommandBufferResetFlags::default())
    }
    .expect("Could not reset command buffer");
    renderer
        .commands
        .begin_command_buffer(&renderer.device.logical);

    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.simulation_image0.image,
        ImageLayout::UNDEFINED,
        ImageLayout::GENERAL,
    );
    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.simulation_image1.image,
        ImageLayout::UNDEFINED,
        ImageLayout::GENERAL,
    );
    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.simulation_image2.image,
        ImageLayout::UNDEFINED,
        ImageLayout::GENERAL,
    );

    run_map_editor(&renderer, command_buffer, uv_min);
    for _ in 0..steps {
        clear_flags(&renderer, command_buffer);
        run_simulation(&renderer, command_buffer);
    }

    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.simulation_image1.image,
        ImageLayout::GENERAL,
        ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.simulation_image2.image,
        ImageLayout::GENERAL,
        ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );

    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.render_image.image,
        ImageLayout::UNDEFINED,
        ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    );

    draw_screen(&renderer, command_buffer);

    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.render_image.image,
        ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ImageLayout::TRANSFER_SRC_OPTIMAL,
    );
    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.images[image_index],
        ImageLayout::UNDEFINED,
        ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    copy_image_to_image(
        &renderer.device,
        command_buffer,
        renderer.swapchain.render_image.image,
        renderer.swapchain.images[image_index],
        renderer.swapchain.extent,
        renderer.swapchain.extent,
    );
    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.images[image_index],
        ImageLayout::TRANSFER_DST_OPTIMAL,
        ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    );
    //draw imgui
    transition_image_layout(
        &renderer.device,
        command_buffer,
        renderer.swapchain.images[image_index],
        ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ImageLayout::PRESENT_SRC_KHR,
    );
    renderer
        .commands
        .end_command_buffer(&renderer.device.logical);

    let command_buffer_submit_infos =
        [CommandBufferSubmitInfo::default().command_buffer(command_buffer)];
    let wait_semaphore_infos = [SemaphoreSubmitInfo::default()
        .semaphore(renderer.commands.get_current_frame().swapchain_semaphore)
        .stage_mask(PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)];
    let signal_semaphore_infos = [SemaphoreSubmitInfo::default()
        .semaphore(renderer.commands.get_current_frame().render_semaphore)
        .stage_mask(PipelineStageFlags2::ALL_GRAPHICS)];
    let submit_infos = [SubmitInfo2::default()
        .command_buffer_infos(&command_buffer_submit_infos)
        .wait_semaphore_infos(&wait_semaphore_infos)
        .signal_semaphore_infos(&signal_semaphore_infos)];

    unsafe {
        renderer.device.logical_sync2.queue_submit2(
            renderer.device.queues.graphics.1,
            &submit_infos,
            renderer.commands.get_current_frame().render_fence,
        )
    }
    .expect("Could not submit queue");
    let swapchains = [renderer.swapchain.handle];
    let image_indices = [image_index as u32];
    let wait_semaphores = [renderer.commands.get_current_frame().render_semaphore];
    let present_info = PresentInfoKHR::default()
        .swapchains(&swapchains)
        .wait_semaphores(&wait_semaphores)
        .image_indices(&image_indices);

    unsafe {
        renderer
            .swapchain
            .loader
            .queue_present(renderer.device.queues.graphics.1, &present_info)
    }
    .expect("Could not present queue");
}

fn clear_flags(renderer: &ResMut<BevyVulkanRenderer>, command_buffer: CommandBuffer) {
    let clear_color = ClearColorValue {
        uint32: [0, 0, 0, 0],
    };
    let clear_range = ImageSubresourceRange::default()
        .aspect_mask(ImageAspectFlags::COLOR)
        .level_count(1)
        .layer_count(1);
    let clear_ranges = [clear_range];
    unsafe {
        renderer.device.logical.cmd_clear_color_image(
            command_buffer,
            renderer.swapchain.simulation_image0.image,
            ImageLayout::GENERAL,
            &clear_color,
            &clear_ranges,
        );
    }
}

fn run_map_editor(
    renderer: &ResMut<BevyVulkanRenderer>,
    command_buffer: CommandBuffer,
    uv_min: Vec2,
) {
    unsafe {
        renderer.device.logical.cmd_bind_pipeline(
            command_buffer,
            PipelineBindPoint::COMPUTE,
            renderer.pipelines.map_editor_pipeline.pipeline,
        );
        let descriptor_sets = [renderer.descriptors.map_editor_descriptor_set];
        renderer.device.logical.cmd_bind_descriptor_sets(
            command_buffer,
            PipelineBindPoint::COMPUTE,
            renderer.pipelines.map_editor_pipeline.pipeline_layout,
            0,
            &descriptor_sets,
            &[],
        );

        let mut push_constants = renderer.pipelines.map_editor_pipeline.data;
        push_constants.update(
            IVec2::new(100, 100) + IVec2::new((uv_min.x * 512.0) as i32, (uv_min.y * 512.0) as i32),
            1,
        );
        let push_constants_bytes: &[u8] = std::slice::from_raw_parts(
            &push_constants as *const MapEditorPushConstants as *const u8,
            size_of::<MapEditorPushConstants>(),
        );
        renderer.device.logical.cmd_push_constants(
            command_buffer,
            renderer.pipelines.map_editor_pipeline.pipeline_layout,
            ShaderStageFlags::COMPUTE,
            0,
            push_constants_bytes,
        );
        renderer
            .device
            .logical
            .cmd_dispatch(command_buffer, 1, 1, 1);
    }
}

fn run_simulation(renderer: &ResMut<BevyVulkanRenderer>, command_buffer: CommandBuffer) {
    unsafe {
        renderer.device.logical.cmd_bind_pipeline(
            command_buffer,
            PipelineBindPoint::COMPUTE,
            renderer.pipelines.simulation_pipeline.pipeline,
        );
        let descriptor_sets = [renderer.descriptors.simulation_descriptor_set];
        renderer.device.logical.cmd_bind_descriptor_sets(
            command_buffer,
            PipelineBindPoint::COMPUTE,
            renderer.pipelines.simulation_pipeline.pipeline_layout,
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
                renderer
                    .device
                    .logical_sync2
                    .cmd_pipeline_barrier2(command_buffer, &dependency_info);
            }
            let mut push_constants = renderer.pipelines.simulation_pipeline.data;
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
            renderer.device.logical.cmd_push_constants(
                command_buffer,
                renderer.pipelines.simulation_pipeline.pipeline_layout,
                ShaderStageFlags::COMPUTE,
                0,
                push_constants_bytes,
            );
            renderer.device.logical.cmd_dispatch(
                command_buffer,
                22, // = 512 / (16)
                22,
                1,
            );
        }
    }
}

fn draw_screen(renderer: &ResMut<BevyVulkanRenderer>, command_buffer: CommandBuffer) {
    let attachment_info = create_rendering_attachment_info(
        renderer.swapchain.render_image.image_view,
        ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        None,
    );
    let color_attachments = [attachment_info];
    let rendering_info = RenderingInfo::default()
        .color_attachments(&color_attachments)
        .layer_count(1)
        .render_area(Rect2D {
            offset: Offset2D::default(),
            extent: renderer.swapchain.extent,
        });
    let viewport = Viewport::default()
        .width(renderer.swapchain.extent.width as f32)
        .height(renderer.swapchain.extent.height as f32);
    let scissor = Rect2D {
        offset: Offset2D::default(),
        extent: renderer.swapchain.extent,
    };

    let push_constants = &renderer.pipelines.render_pipeline.data;

    unsafe {
        let push_constants_bytes: &[u8] = std::slice::from_raw_parts(
            push_constants as *const PushConstants as *const u8,
            size_of::<PushConstants>(),
        );
        let logical_device = &renderer.device.logical;
        let logical_device_dyn = &renderer.device.logical_dynamic_rendering;
        logical_device_dyn.cmd_begin_rendering(command_buffer, &rendering_info);
        logical_device.cmd_bind_pipeline(
            command_buffer,
            PipelineBindPoint::GRAPHICS,
            renderer.pipelines.render_pipeline.pipeline,
        );
        let descriptor_sets = [renderer.descriptors.render_descriptor_set];
        renderer.device.logical.cmd_bind_descriptor_sets(
            command_buffer,
            PipelineBindPoint::GRAPHICS,
            renderer.pipelines.render_pipeline.pipeline_layout,
            0,
            &descriptor_sets,
            &[],
        );
        renderer.device.logical.cmd_push_constants(
            command_buffer,
            renderer.pipelines.render_pipeline.pipeline_layout,
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

fn create_rendering_attachment_info(
    view: ImageView,
    layout: ImageLayout,
    clear: Option<ClearValue>,
) -> RenderingAttachmentInfo<'static> {
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

fn handle_window_resize(
    mut renderer: ResMut<BevyVulkanRenderer>,
    mut resize_events: EventReader<WindowResized>,
) {
    for e in resize_events.read(){
        renderer.recreate_swap_chain();
    }
}