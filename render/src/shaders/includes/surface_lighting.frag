
vec3 lighting = vec3(0.0, 0.0, 0.0);

// Ambient lights
vec3 ambient = vec3(0, 0, 0);
for (int i = 0; i < ambient_lights.data.length(); i++) {
    AmbientLight light = ambient_lights.data[i];
    ambient += light.color * light.intensity;
}
lighting += ambient;

// Directional lights
vec3 directional = vec3(0, 0, 0);
for (int i = 0; i < directional_lights.data.length(); i++) {
    DirectionalLight light = directional_lights.data[i];
    
    directional += light.color * dot(normal, -light.direction) * light.intensity;
}
lighting += directional;

// Point lights
vec3 point = vec3(0, 0, 0);
for (int i = 0; i < point_lights.data.length(); i++) {
    PointLight light = point_lights.data[i];

    vec3 dir_to_light = light.position - position;        
    float dist_to_light = length(dir_to_light);

    point += light.color * dot(normal, normalize(dir_to_light)) * light.intensity / pow(dist_to_light, 2);
}
lighting += point;