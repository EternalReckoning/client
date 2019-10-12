#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 1) uniform texture2D colormap;
layout(set = 0, binding = 2) uniform sampler colorsampler;

void main() {
    color = texture(sampler2D(colormap, colorsampler), uv);
}