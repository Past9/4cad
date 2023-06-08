#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) flat in uint material_idx;

#include "surface_lighting_buffers.frag"
#include "translucent_material_buffer.frag"

layout(input_attachment_index = 1, set = 0, binding = 4) uniform subpassInputMS u_depth;

layout(location = 0) out vec4 total_reflection;
layout(location = 1) out vec3 total_transmission;

void computeOutput(
    vec3 reflection, 
    float alpha, 
    vec3 transmission
) {
    float netCoverage = alpha * (1.0 - dot(transmission, vec3(1.0 / 3.0)));
    float tmp = (1.0 - gl_FragCoord.z * 0.99) * netCoverage * 10.0;
    float weight = clamp(tmp * tmp * tmp, 0.01, 30.0);

    total_reflection = vec4(reflection * alpha, netCoverage) * weight;
    total_transmission = alpha * (vec3(1.0) - transmission);
}

void main() {
    float self_depth = gl_FragCoord.z;
    float in_depth = subpassLoad(u_depth, gl_SampleID).x;

    if (in_depth < self_depth) {
        discard;
    }

    Material material = materials.data[material_idx];

    #include "surface_lighting.frag"

    vec3 reflected = material.diffuse.rgb * lighting;

    computeOutput(
        reflected,
        material.diffuse.a,
        material.diffuse.rgb * (1.0 - material.diffuse.a)
    );
}