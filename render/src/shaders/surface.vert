#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(push_constant) uniform PushConstants {
    mat4 view;
    mat4 model;
    mat4 perspective;
} push_constants;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec3 v_position;

void main() {
    mat4 model_view = push_constants.view * push_constants.model;
    gl_Position = push_constants.perspective * model_view * vec4(position, 1.0);

    v_normal = transpose(inverse(mat3(model_view))) * normal;
    v_position = vec3(model_view * vec4(position, 1.0));
}