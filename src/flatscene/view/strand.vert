#version 450

layout(set = 0, binding = 0)
uniform Globals {
    vec2 u_resolution;
    vec2 u_scroll_offset;
    float u_zoom;
};

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_normal;
layout(location = 2) in vec4 a_color;
layout(location = 3) in float a_depth;
layout(location = 4) in float a_width;


layout(location = 0) out vec4 v_color;

void main() {
    vec2 invert_y = vec2(1.0, -1.0);

    vec2 local_pos = a_position + a_normal * 0.1125 * a_width;
    vec2 world_pos = local_pos - u_scroll_offset;
    vec2 transformed_pos = world_pos * u_zoom / (vec2(0.5, 0.5) * u_resolution) * invert_y;

// formula for the grid:
//float z = (float(model.z_index * 1000 + a_model_id) + background_depth);
// We want the strand to have the depth of the grid - 0.25.

    float z = a_depth * 1000.;
    gl_Position = vec4(transformed_pos, z / 1e6, 1.0);
    v_color = a_color;
}
