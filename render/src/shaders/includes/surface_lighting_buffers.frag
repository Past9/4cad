struct PointLight {
    vec3 position;
    vec3 color;
    float intensity;
};

struct AmbientLight {
    vec3 color;
    float intensity;
};

struct DirectionalLight {
    vec3 direction;
    vec3 color;
    float intensity;
};

layout(std140, set = 0, binding = 0) readonly buffer PointLightBuffer {
    PointLight data[];
} point_lights;

layout(std140, set = 0, binding = 1) readonly buffer AmbientLightBuffer {
    AmbientLight data[];
} ambient_lights;

layout(std140, set = 0, binding = 2) readonly buffer DirectionalLightBuffer {
    DirectionalLight data[];
} directional_lights;