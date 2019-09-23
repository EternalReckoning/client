#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
// vec4[4] is used instead of mat4 due to spirv-cross bug
layout(location = 2) in vec4 model[4];
layout(location = 0) out vec4 frag_color;

layout(set = 0, binding = 0) uniform Args {
    mat4 proj;
    mat4 view;
};

void main() {
    mat4 model_mat = mat4(model[0], model[1], model[2], model[3]);
    frag_color = color;
    gl_Position = proj * view * model_mat * vec4(position, 1.0);
}