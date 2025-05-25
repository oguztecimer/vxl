//#version 450
//layout(location = 0) out vec4 color;
//
//layout(set = 0, binding = 0) uniform sampler2D images[6]; // Sample input images (0: R8G8B8A8_UINT)
//
//#define GRID_WIDTH 512
//#define GRID_HEIGHT 512
//
//void main() {
//    // Window resolution (passed via push constant or uniform)
//    float window_width = 1920.0; // Example
//    float window_height = 1080.0;
//
//    // Pixel-perfect scaling
//    float scale = floor(min(window_width / GRID_WIDTH, window_height / GRID_HEIGHT)); // Integer scale (e.g., 2.0)
//    float grid_scaled_width = GRID_WIDTH * scale;
//    float grid_scaled_height = GRID_HEIGHT * scale;
//
//    // Center grid in window
//    float offset_x = (window_width - grid_scaled_width) * 0.5;
//    float offset_y = (window_height - grid_scaled_height) * 0.5;
//
//    // Map window UV to grid UV
//    vec2 grid_uv = (uv * vec2(window_width, window_height) - vec2(offset_x, offset_y)) / vec2(grid_scaled_width, grid_scaled_height);
//
//    // Check if UV is within grid bounds
//    if (grid_uv.x < 0.0 || grid_uv.x > 1.0 || grid_uv.y < 0.0 || grid_uv.y > 1.0) {
//        color = vec4(0.0, 0.0, 0.0, 1.0); // Black bars
//        return;
//    }
//
//    // Sample grid texture (nearest for pixelated look)
//    ivec2 pos = ivec2(grid_uv * vec2(GRID_WIDTH, GRID_HEIGHT));
//    uint material = texelFetch(images[0], pos, 0).x;
//
//    // Map material to color (example)
//    vec3 colors[3] = {vec3(0.0), vec3(0.5, 0.5, 0.5), vec3(0.0, 0.0, 1.0)}; // Air, Stone, Water
//    color = vec4(colors[material % 3], 1.0);
//}



#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform sampler2D image;

//layout( push_constant ) uniform constants
//{
//    vec2 offset;
//    float scale;
//} PushConstants;


void main() {
    //color = vec4(1.0,0.0,0.0,1.0);
    color = texture(image,uv)*5.0;
}