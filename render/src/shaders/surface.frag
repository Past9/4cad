#version 450

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec3 v_position;

const vec3 light_dir = vec3(1.0, -1.0, -1.0);
const vec3 ambient_color = vec3(0.2, 0.2, 0.2);
const vec3 diffuse_color = vec3(0.6, 0.6, 0.6);
const vec3 specular_color = vec3(1.0, 1.0, 1.0);
const float specular_strength = 1.0;
const float shininess = 16.0;

layout(location = 0) out vec4 f_color;

void main() {
    vec3 norm = normalize(v_normal);
    vec3 frag_pos = v_position;

    // Translate the light direction into the correct coordinate system 
    vec3 light_dir = normalize(vec3(light_dir.x, -light_dir.y, -light_dir.z));
    vec3 view_pos = vec3(0, 0, 0);

    float diff = max(dot(norm, light_dir), 0.0);
    vec3 diffuse = diff * diffuse_color;

    vec3 view_dir = normalize(view_pos - frag_pos);
    vec3 reflect_dir = reflect(-light_dir, norm);

    float spec = pow(max(dot(view_dir, reflect_dir), 0.0), shininess);
    vec3 specular = specular_strength * spec * specular_color;

    vec3 result = (ambient_color + diffuse + specular);

    f_color = vec4(result, 1.0);
}