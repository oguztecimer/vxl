#version 460

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform usampler2D image1;
layout(set = 0, binding = 1) uniform sampler2D image2;


layout( push_constant ) uniform constants
{
    vec2 uv_min;
    vec2 uv_max;
    ivec2 batch_offset;
    uint frame_number;

} PushConstants;


void main() {
    uvec4 properties1;
    vec2 mappedUv = mix(PushConstants.uv_min,PushConstants.uv_max, uv);
    properties1 = texture(image1,mappedUv);
    if (properties1.r>0){
        float variant = float(properties1.g)/50;
        color = vec4(1.0-variant,1.0-variant,0.0,0.0);
    }else{
        color = vec4(0.0,0.0,0.0,0.0);
    }
}
