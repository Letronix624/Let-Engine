#version 450

layout (location = 0) in vec2 position;
layout (location = 0) out int obj1;
layout (location = 1) out vec4 vertex_color;
layout (set = 0, binding = 1) buffer Data {
    vec2 position;
    vec2 size;
    float rotation;
} player;

vec4 orange = vec4(1.00, 0.50, 0.00, 1.0);
vec4 lavender = vec4(0.53, 0.4, 0.8, 1.0);
vec4 skyblue = vec4(0.10, 0.40, 1.00, 1.0);
vec4 maya = vec4(0.5, 0.75, 1.0, 0.8);


vec4 colors[] = vec4[](
    lavender, // top left
    orange, // bottom left
    lavender, // top right
    // vertex 2
    orange, // bottom left
    orange, // bottom right
    lavender, // top right
    // upper row
    skyblue, // top left
    lavender, // bottom left
    skyblue, // top right
    // vertex 2
    lavender, // bottom left
    lavender, // bottom right
    skyblue // top right
);
void main() {
    // vec2 pos = obj.position;
    // float rot = obj.rotation;

    gl_Position = vec4(position * player.size, 0.0, 1.0);
    vertex_color = colors[gl_VertexIndex];
    obj1 = 0;
    if (colors.length() <= gl_VertexIndex){
        obj1 = 1;
    }
}
