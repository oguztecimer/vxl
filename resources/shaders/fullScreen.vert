#version 450
layout(location = 0) out vec2 uv;

void main() {
    // Generate uv in [0, 1] using bitwise operations
    uv = vec2((gl_VertexIndex & 2) >> 1, (gl_VertexIndex & 1));
    // Map uv [0, 1] to clip space [-1, 1]
    gl_Position = vec4((uv * 2.0 - 1.0), 0.0, 1.0);
}