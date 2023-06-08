#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) flat in uint material_idx;

#include "surface_lighting_buffers.frag"
#include "opaque_material_buffer.frag"

layout(location = 0) out vec4 color;

void main() {
    Material material = materials.data[material_idx];
    color = vec4(material.diffuse, 1.0);

    #include "surface_lighting.frag"

    color.rgb *= lighting;
}