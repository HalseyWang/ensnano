// shader.frag
#version 450

layout(location=0) in vec4 v_color;
layout(location=1) in vec2 v_tex_coords;

layout(location=0) out vec4 f_color;

layout(set = 2, binding = 0) uniform texture2D t_diffuse;
layout(set = 2, binding = 1) uniform sampler s_diffuse;


void main() {
    vec4 color  =  texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
    if (color.w < 0.01) {
    discard;
    }

    f_color =  vec4(vec3(v_color * color), 1.);
}
