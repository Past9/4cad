#version 450

layout(input_attachment_index = 1, set = 0, binding = 0) uniform subpassInputMS u_opaque;
layout(input_attachment_index = 1, set = 0, binding = 1) uniform subpassInputMS u_accum;
layout(input_attachment_index = 1, set = 0, binding = 2) uniform subpassInputMS u_transmit;

layout(location = 0) out vec4 color;

float minComponent(vec3 vec) {
    return min(min(vec.x, vec.y), vec.z);
}

float maxComponent(vec3 vec) {
    return max(max(vec.x, vec.y), vec.z);
}

void main() {
    vec4 opaque = subpassLoad(u_opaque, gl_SampleID);
    vec4 accum = subpassLoad(u_accum, gl_SampleID);
    vec4 transmit = subpassLoad(u_transmit, gl_SampleID);
    //color = opaque * vec4(transmit.rgb, 1.0) + accum;

    /*
    vec3 bkg = opaque.rgb;
    vec3 B = transmit.rgb;
    vec4 A = accum;

    vec3 color_rgb = bkg * B + (vec3(1) - B) * A.rgb / max(A.a, 0.00001);
    color = vec4(color_rgb, 1.0);
    */

    //color = vec4(opaque.rgb * transmit.rgb + (vec3(1) - transmit.rgb) * accum.rgb / max(accum.a, 0.00001), 1.0);

    vec3 B = subpassLoad(u_transmit, gl_SampleID).rgb;

    if (minComponent(B) == 1.0) {
        color = opaque;
        return;
    }

    vec4 A = subpassLoad(u_accum, gl_SampleID);

    if (isinf(A.a)) A.a = maxComponent(A.rgb);
    if (isinf(maxComponent(A.rgb))) A = vec4(isinf(A.a) ? 1.0 : A.a);

    A.rgb *= vec3(0.5) + 0.5 * B / max(0.01, maxComponent(B));

    vec3 bkg = subpassLoad(u_opaque, gl_SampleID).rgb;

    vec3 color_rgb = bkg * B + (vec3(1) - B) * A.rgb / max(A.a, 0.00001);
    color = vec4(color_rgb, 1.0);

    //color = vec4(0.5, 0.5, 0.5, 1.0);

}