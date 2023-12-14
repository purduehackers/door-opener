use macroquad::{prelude::*, miniquad::{BlendState, Equation, BlendFactor, BlendValue}};

use super::svg;

const NOISE_SVG: &[u8] = include_bytes!("./noise.svg");
const PASSPORT_EMBLEM: &[u8] = include_bytes!("./passport-emblem.svg");

pub struct PassportData {
    material: Material,
    logo_texture: Texture2D,
    noise_texture: Texture2D
}

pub async fn initialise_passport() -> PassportData {
    let logo_texture = svg::svg_to_texture(String::from_utf8(PASSPORT_EMBLEM.to_vec()).unwrap().as_str());
    let noise_texture = svg::svg_to_texture(String::from_utf8(NOISE_SVG.to_vec()).unwrap().as_str());

    let passport_material = load_material(
        ShaderSource::Glsl {
            vertex: PASSPORT_VERTEX_SHADER,
            fragment: PASSPORT_FRAGMENT_SHADER,
        },
        MaterialParams {
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()  
            },
            uniforms: vec![
                ("time".to_owned(), UniformType::Float1)
            ],
            textures: vec![
                "logo_texture".to_string(),
                "noise_texture".to_string()
            ],
            ..Default::default()
        },
    )
    .unwrap();

    return PassportData {
        material: passport_material,
        logo_texture: logo_texture,
        noise_texture: noise_texture
    };
}

pub fn draw_passport(x: f32, y: f32, _state: u8, passport_data: &PassportData) {
    passport_data.material.set_uniform("time", (get_time() % 6.0) as f32);
    passport_data.material.set_texture("logo_texture", passport_data.logo_texture.to_owned());
    passport_data.material.set_texture("noise_texture", passport_data.noise_texture.to_owned());

    gl_use_material(&passport_data.material);
    draw_rectangle(x, y, 332.0, 472.0, WHITE);
    gl_use_default_material();
}

const PASSPORT_VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;
varying vec2 screen_position;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    uv = texcoord;

    gl_Position = Projection * Model * vec4(position, 1);
    screen_position = vec2(position.x, 720.0 - position.y);
}
";

const PASSPORT_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 uv;
varying vec2 screen_position;

uniform float time;
uniform sampler2D logo_texture;
uniform sampler2D noise_texture;

uniform sampler2D _ScreenTexture;

float normpdf(in float x, in float sigma)
{
	return 0.39894*exp(-0.5*x*x/(sigma*sigma))/sigma;
}

void main() {
    const int mSize = 48;
	const int kSize = (mSize - 1) / 2;
	float kernel[mSize];
	vec3 final_colour = vec3(0.0);
	
	float sigma = 14.0;
	float Z = 0.0;
	for (int j = 0; j <= kSize; ++j)
	{
		kernel[kSize + j] = kernel[kSize - j] = normpdf(float(j), sigma);
	}
	
	for (int j = 0; j < mSize; ++j)
	{
		Z += kernel[j];
	}
	
	for (int i = -kSize; i <= kSize; ++i)
	{
		for (int j = -kSize; j <= kSize; ++j)
		{
			final_colour += kernel[kSize + j] * kernel[kSize + i] * texture2D(_ScreenTexture, (screen_position + vec2(float(i), float(j))) / vec2(720.0, 720.0)).rgb;
        }
	}
	
	gl_FragColor = vec4(final_colour / (Z * Z), 1.0) + (texture2D(noise_texture, uv) * 0.15 - 0.075);
}
";