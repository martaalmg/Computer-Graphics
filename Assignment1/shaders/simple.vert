#version 430 core

in vec3 position;

uniform mat4 mirror_scale= mat4(
    -1, 0, 0, 0,
    0, -1, 0, 0,
    0, 0, 1, 0,
    0, 0, 0, 1);

void main()
{
    gl_Position = mirror_scale*vec4(position, 1.0f);
}