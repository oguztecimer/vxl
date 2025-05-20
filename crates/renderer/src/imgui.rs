use ash::Instance;
use ash::vk::{CommandPool, Format};
use imgui::{Context, FontSource};
use imgui_rs_vulkan_renderer::{DynamicRendering, Options, Renderer};
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use crate::device::Device;
pub fn setup_imgui(window: &winit::window::Window) -> (Context, WinitPlatform) {
    let mut imgui = Context::create();
    imgui.set_ini_filename(None); // Optional: Disable saving ImGui settings

    let mut platform = WinitPlatform::new(&mut imgui);
    platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);

    // Configure ImGui style and fonts (optional)
    imgui
        .fonts()
        .add_font(&[FontSource::DefaultFontData { config: None }]);
    imgui.io_mut().font_global_scale = 1.0;

    (imgui, platform)
}

pub fn create_imgui_renderer(
    instance: &Instance,
    device: &Device,
    imgui: &mut Context,
    options: Option<Options>,
) -> (Renderer, CommandPool) {
    let command_pool =
        crate::commands::Commands::create_command_pool(&device.logical, device.queues.graphics.0);
    //let renderer = Renderer::with_vk_mem_allocator(
    let renderer = Renderer::with_default_allocator(
        instance,
        device.physical,
        device.logical.clone(),
        device.queues.graphics.1,
        command_pool,
        DynamicRendering {
            color_attachment_format: Format::B8G8R8A8_UNORM,
            depth_attachment_format: None,
        },
        imgui,
        options,
    )
    .expect("Could not create imgui renderer");
    (renderer, command_pool)
}
