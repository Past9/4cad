#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(push_constant) uniform PushConstants {
    mat4 model_matrix;
    mat4 projection_matrix;
} push_constants;

layout(location = 0) out vec4 v_color;

void main() {
    gl_Position = push_constants.projection_matrix * push_constants.model_matrix * vec4(position, 1.0);
    v_color = color;
}