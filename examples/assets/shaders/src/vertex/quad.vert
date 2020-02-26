#version 450


layout(std140, set = 0, binding = 0) uniform Projview {
    mat4 proj;
    mat4 view;
    mat4 proj_view;
};


layout(location = 0) in vec3 position;
layout(location = 1) in vec3 translate;
layout(location = 2) in uint dir;
layout(location = 3) in vec3 color;
layout(location = 4) in uint pad;



layout(location = 0) out VertexData {
    vec4 pos;
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
   
    vertex.color = vec4(color, 1.0);
    // frag_norm = normalize((vec4(normal, 1.0)).xyz);
    mat4 trans_mat = mat4(1.0);
    trans_mat[3] = vec4(translate, 1.0);
    mat4 model2 = trans_mat * model[dir];
    vertex.pos = vec4(position, 1.0);
    gl_Position = proj * view * model2 * frag_pos;

}
