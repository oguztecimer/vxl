#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform usampler2D properties1_in;
layout(set = 0, binding = 1) uniform usampler2D properties1_out;
layout(set = 0, binding = 2) uniform usampler2D properties2_in;
layout(set = 0, binding = 3) uniform usampler2D properties2_out;

uvec4 properties1;
uvec4 load_properties1(vec2 uv);

layout( push_constant ) uniform constants
{
    uint swap_io;
    float delta_time;
    vec2 uv_min;
    vec2 uv_max;
} PushConstants;


void main() {
    vec2 mappedUv = mix(PushConstants.uv_min,PushConstants.uv_max, uv);
    properties1 = load_properties1(mappedUv);
    if (properties1.r>0){
        color = vec4(1.0,1.0,0.0,0.0);
    }else{
        color = vec4(0.0,0.0,0.0,0.0);
    }
}

uvec4 load_properties1(vec2 scaled_uv){
    if (PushConstants.swap_io == 0) {
        return texture(properties1_out, scaled_uv);
    } else {
        return texture(properties1_in, scaled_uv);
    }
}