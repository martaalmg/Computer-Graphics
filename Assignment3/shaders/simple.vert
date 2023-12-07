#version 430 core


// Declare in the location of each variable we are going to use

layout(location = 0 ) in vec3 position;

// To store the colors
layout(location = 1) in vec4 in_color;
layout(location = 1) out vec4 out_color;

uniform layout(location = 2) mat4 transf_matrix; // used to transform the vertex positions

// To store the normals
layout(location = 3) in vec3 in_normals;
layout(location = 3) out vec3 out_normals;

uniform layout(location = 4) mat4 model_matrix; // used to transform the vertex normals


void main()
{
    out_color = in_color;

    out_normals = normalize(vec3(model_matrix * vec4(in_normals, 0.0))); // normalize the result

    gl_Position = transf_matrix * vec4(position, 1.0f);

}