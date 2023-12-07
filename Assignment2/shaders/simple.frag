#version 430 core

layout(location=1) in vec4 in_color;
out vec4 color;

void main()
{
    color = in_color;
}