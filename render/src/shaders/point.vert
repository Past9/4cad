#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 expand;

layout(push_constant) uniform PushConstants {
    mat4 view;
    mat4 model;
    mat4 perspective;
} push_constants;

// This shader is used for rendering edges from line lists. Since these edges
// generally use the same vertices as the triangles ( or at least lie in the 
// same plane), z-fighting is an issue, and PolygonOffset is an unreliable
// solution.
//
// To solve this, we take the lines and move them outward slightly along their 
// expand vector (by taking the vertex position and adding the expand vector,
// multiplied by a very small hand-tuned factor). This is done before any 
// transforms (including perspective) are applied.
//
// This approach results in lines that are "above" the surfaces on which 
// they lie by just enough to overcome z-fighting. One problem with it is 
// that if the user zooms in very closely, the gap between line and surface 
// is visible. Ideally, we want to shrink the gap as the camera gets closer
// to it. We can do this by using the z-distance of the vertex from 
// the camera as another factor by which to multiply the expand vector.
// However, we don't know the z-distance because we haven't done any of 
// the transformations yet. Therefore it is necessary to transform the 
// original vertex, get its z-distance, use that as a factor to adjust 
// the vertex's position, and then apply the transformation again to the 
// adjusted vertex. This second transformation is what results in the final 
// gl_Position.

// Assumptions: 
//  - The provided expand vector points away from the surface on which the 
//    un-adjusted vertex lies.
//  - The camera zooms by moving closer to the object. If zooming is done by
//    scaling the object or narrowing the camera's field of view, or an 
//    orthographic camera is used, the z-shrinking may not work.
void main() {
    // Create the view matrix
    mat4 modelview = push_constants.view * push_constants.model;

    // Calculate the final position of the vertex if we don't adjust it. Don't do 
    // the perspective transformation here--we want a raw z-distance from the camera.
    vec4 unadjusted_transformed_position = modelview * vec4(position, 1.0);

    // Use that position's z-distance from the camera, multiplied by a hand-tuned 
    // factor, as a distance to move the vertex along its expand vector. 
    float offset = -unadjusted_transformed_position.z / 2000;
    vec3 adjusted_position = position + expand * offset;

    // Finally, apply the projection/model/view transformations to the adjusted
    // vertex.
    gl_Position = push_constants.perspective * modelview * vec4(adjusted_position, 1.0);

    // Large points
    gl_PointSize = 10.0;
}