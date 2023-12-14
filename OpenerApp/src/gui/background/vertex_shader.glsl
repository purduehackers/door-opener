#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying vec2 pixel_coord;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    pixel_coord = texcoord * vec2(720.0, 720.0);

    gl_Position = Projection * Model * vec4(position, 1);
}