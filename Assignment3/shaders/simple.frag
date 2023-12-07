#version 430 core

layout(location=1) in vec4 in_color;
layout(location=3) in vec3 in_normals;
out vec4 out_color;

void main()
{
    vec3 lightDir = normalize(vec3(0.8, -0.5, 0.6));
    vec3 color_lights = vec3(in_color[0], in_color[1], in_color[2]) * max(dot(in_normals, -lightDir), 0);
    out_color = vec4(color_lights[0], color_lights[1], color_lights[2], in_color[3]);

}