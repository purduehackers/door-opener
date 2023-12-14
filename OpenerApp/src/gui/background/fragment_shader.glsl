#version 100
precision lowp float;

varying vec2 pixel_coord;

uniform float time;
uniform sampler2D logo_texture;

void main() {
    gl_FragColor = texture2D(logo_texture, mod(((mod(pixel_coord, 100.0)) / 100.0) + (vec2(1, -0.5) * (mod(time, 6.0) / 3.0)), vec2(1.0, 1.0)));
}