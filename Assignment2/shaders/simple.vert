#version 430 core

//task 3: modify so that each vertex is multipled by a 4x4 ma
//identity matrix
//render from 1b

layout(location = 0 ) in vec3 position;
layout(location = 1) in vec4 in_color;
layout(location = 1) out vec4 out_color;
uniform layout(location = 2) mat4 transf_matrix;

// a b 0 c
// d e 0 f
// 0 0 1 0
// 0 0 0 1

//order of manipulation: 3b
// a -1 e -1
//c d 1
//c d -1
// a e 1.5


//float a = 1.0f;
//float e = 1.0f;
//float b = 0.0f;
//float c = 0.0f;
//float d = 0.0f;
//float f = 0.0f;

//should pass matrix as ptr

//mat4 identity_matrix = mat4(
  //  a, b, 0, c,
    //d, e, 0, f,
    //0, 0, 1, 0,
    //0, 0, 0, 1
//);

void main()
{
    gl_Position = transf_matrix*vec4(position, 1.0f);
    out_color = in_color;
}