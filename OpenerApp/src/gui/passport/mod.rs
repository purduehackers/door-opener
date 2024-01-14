use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};

use super::svg;

const NOISE_SVG: &[u8] = include_bytes!("../noise.svg");
const PASSPORT_EMBLEM: &[u8] = include_bytes!("../passport-emblem.svg");
const LOADING_SPINNER: &[u8] = include_bytes!("../loading-spinner.svg");

pub struct PassportData {
    material: Material,
    logo_texture: Texture2D,
    noise_texture: Texture2D,
    loading_spinner_texture: Texture2D,
    current_spinner_colour: Color,
    current_spinner_cutout_opacity: f32,
    current_x: f32,
    current_y: f32,
}

pub async fn initialise_passport() -> PassportData {
    let logo_texture = svg::svg_to_texture(
        String::from_utf8(PASSPORT_EMBLEM.to_vec())
            .unwrap()
            .as_str(),
    );
    let noise_texture =
        svg::svg_to_texture(String::from_utf8(NOISE_SVG.to_vec()).unwrap().as_str());
    let loading_spinner_texture = svg::svg_to_texture(
        String::from_utf8(LOADING_SPINNER.to_vec())
            .unwrap()
            .as_str(),
    );

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
            uniforms: vec![("time".to_owned(), UniformType::Float1)],
            textures: vec!["logo_texture".to_string(), "noise_texture".to_string()],
            ..Default::default()
        },
    )
    .unwrap();

    return PassportData {
        material: passport_material,
        logo_texture: logo_texture,
        noise_texture: noise_texture,
        loading_spinner_texture: loading_spinner_texture,
        current_spinner_colour: Color::from_hex(0xfbcb3b),
        current_spinner_cutout_opacity: 0.0,
        current_x: 0.0,
        current_y: 0.0,
    };
}

pub fn draw_passport(x: f32, y: f32, state: i32, passport_data: &mut PassportData) {
    passport_data
        .material
        .set_uniform("time", (get_time() % 6.0) as f32);
    passport_data
        .material
        .set_texture("logo_texture", passport_data.logo_texture.to_owned());
    passport_data
        .material
        .set_texture("noise_texture", passport_data.noise_texture.to_owned());

    let delta_time: f32 = get_frame_time();
    let loading_spinner_angle = (get_time() * 3.0) as f32;

    if state == 0 {
        passport_data.current_x = x;
        passport_data.current_y = y;
    } else {
        let animation_speed = match state {
            0 => 5.0,
            1 => 5.0,
            2 => 2.0,
            3 => 5.0,
            _default => 5.0,
        };

        passport_data.current_x =
            super::float32_lerp(passport_data.current_x, x, delta_time * animation_speed);
        passport_data.current_y =
            super::float32_lerp(passport_data.current_y, y, delta_time * animation_speed);
    }

    gl_use_material(&passport_data.material);
    draw_rectangle(
        passport_data.current_x - 166.0,
        passport_data.current_y - 236.0,
        332.0,
        472.0,
        WHITE,
    );
    gl_use_default_material();

    let center_x = passport_data.current_x - (passport_data.logo_texture.width() / 2.0);
    let center_y = passport_data.current_y - (passport_data.logo_texture.height() / 2.0);

    passport_data.current_spinner_cutout_opacity = super::float32_lerp(
        passport_data.current_spinner_cutout_opacity,
        match state {
            0 => 1.0,
            1 => 0.0,
            2 => 1.0,
            3 => 1.0,
            _default => 1.0,
        },
        delta_time * 10.0,
    );
    passport_data.current_spinner_colour = super::colour_lerp(
        passport_data.current_spinner_colour,
        match state {
            0 => Color::from_hex(0xfbcb3b),
            1 => Color::from_hex(0xfbcb3b),
            2 => Color::from_hex(0x22c55e),
            3 => Color::from_hex(0xef4444),
            _default => Color::from_hex(0xfbcb3b),
        },
        delta_time * 10.0,
    );

    draw_texture(
        &passport_data.logo_texture,
        center_x,
        center_y,
        Color::from_hex(0xfbcb3b),
    );

    draw_texture_ex(
        &passport_data.loading_spinner_texture,
        center_x,
        center_y,
        Color {
            r: passport_data.current_spinner_colour.r,
            g: passport_data.current_spinner_colour.g,
            b: passport_data.current_spinner_colour.b,
            a: passport_data.current_spinner_cutout_opacity,
        },
        DrawTextureParams {
            dest_size: Option::None,
            source: Option::None,
            rotation: loading_spinner_angle,
            flip_x: true,
            flip_y: true,
            pivot: Option::None,
        },
    );

    draw_texture_ex(
        &passport_data.loading_spinner_texture,
        center_x,
        center_y,
        passport_data.current_spinner_colour,
        DrawTextureParams {
            dest_size: Option::None,
            source: Option::None,
            rotation: loading_spinner_angle,
            flip_x: false,
            flip_y: false,
            pivot: Option::None,
        },
    );
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
precision highp float;

varying vec2 uv;
varying vec2 screen_position;

uniform float time;
uniform sampler2D logo_texture;
uniform sampler2D noise_texture;

uniform sampler2D _ScreenTexture;

#define RADIUS 12.0

float normpdf(in float x, in float sigma)
{
	return 0.39894*exp(-0.5*x*x/(sigma*sigma))/sigma;
}

float rounded_box_signed_distance_factor(vec2 CenterPosition, vec2 Size, float Radius) {
    return length(max(abs(CenterPosition)-Size+Radius,0.0))-Radius;
}

void main() {
    const int mSize = 52;
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
	
	for (int i = -kSize; i <= kSize; i += 2)
	{
		for (int j = -kSize; j <= kSize; j += 2)
		{
			final_colour += (kernel[kSize + j] * kernel[kSize + i] * texture2D(_ScreenTexture, (screen_position + vec2(float(i), float(j))) / vec2(720.0, 720.0)).rgb) * 3.0;
        }
	}

	vec4 mixed_colour = vec4(final_colour / (Z * Z), 1.0) + (texture2D(noise_texture, uv) * 0.15 - 0.05);

    float distance = rounded_box_signed_distance_factor((uv * vec2(332.0, 472.0)) - (vec2(332.0, 472.0) / 2.0), vec2(332.0, 472.0) / 2.0, RADIUS);

    float smoothedAlpha = 1.0 - smoothstep(0.0, 2.0, distance);

    vec4 quadColour = mix(vec4(texture2D(_ScreenTexture, screen_position).rgb, 0.0), vec4(mixed_colour.rgb, smoothedAlpha), smoothedAlpha);
    
    gl_FragColor = quadColour;
}
";