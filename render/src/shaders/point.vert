#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 expand;
layout(location = 2) in vec4 color;

layout(push_constant) uniform PushContstants {
    mat4 model_matrix;
    mat4 projection_matrix;
} push_constants;

layout(location = 0) out vec4 v_color;

void main() {
    vec4 unadjusted_transformed_pos = push_constants.model_matrix * vec4(position, 1.0);
    
    float offset = -unadjusted_transformed_pos.z / 2000;
    vec3 adjusted_pos = position + expand * offset;

    gl_Position = push_constants.projection_matrix * push_constants.model_matrix * vec4(adjusted_pos, 1.0);

    gl_PointSize = 10.0;

    v_color = color;
}