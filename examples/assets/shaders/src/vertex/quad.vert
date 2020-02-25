#version 450


layout(std140, set = 0, binding = 0) uniform Projview {
    mat4 proj;
    mat4 view;
    mat4 proj_view;
};


layout(location = 0) in vec2 pos;
layout(location = 1) in vec4 color;

layout(location = 0) out VertexData {
    vec2 pos;
    vec4 color;
} vertex;


void main() {
    mat4 model0 = mat4(0.125, 0.0, 0.0, 0.0, 0.0, 0.125, 0.0, 0.0, 0.0, 0.0, 0.125, 0.0, 0.0, 0.0, 0.125, 1.0);
    mat4 model1 = mat4(-0.125, 0.0, 0.0, 0.0, 0.0, 0.125, 0.0, 0.0, 0.0, 0.0, -0.125, 0.0, 0.0, 0.0, -0.125, 1.0);
    mat4 model2 = mat4(0.0, 0.0, -0.125, 0.0, 0.0, 0.125, 0.0, 0.0, 0.125, 0.0, 0.0, 0.0, 0.125, 0.0, 0.0, 1.0);
    mat4 model3 = mat4(0.0, -0.0, 0.125, 0.0, 0.0, 0.125, 0.0, 0.0, -0.125, 0.0, 0.0, 0.0, -0.125, 0.0, 0.0, 1.0);
    mat4 model4 = mat4(-0.125, 0.0, 0.0, 0.0, 0.0, 0.0, 0.125, 0.0, 0.0, 0.125, 0.0, 0.0, 0.0, 0.125, 0.0, 1.0);
    mat4 model5 = mat4(-0.125, -0.0, 0.0, 0.0, 0.0, 0.0, -0.125, 0.0, 0.0, -0.125, 0.0, 0.0, 0.0, -0.125, 0.0, 1.0);
    mat4 model[6] = mat4[6](model0, model1, model2, model3, model4, model5);
   
    vertex.pos = pos;
    vertex.color = color;

    vec4 position = vec4(pos, 0.0, 1.0);
    gl_Position = proj_view * position;
}
