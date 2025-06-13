use bevy::prelude::*;
use bevy_vulkan_renderer::plugin::BevyVulkanPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyVulkanPlugin)
        .run();
}
