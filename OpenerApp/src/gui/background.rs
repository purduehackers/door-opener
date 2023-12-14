use macroquad::{prelude::*, miniquad::{BlendState, Equation, BlendFactor, BlendValue}};

const PH_LOGO_TILABLE: &[u8] = include_bytes!("./ph-logo-tilable.png");

pub struct BackgroundData {
    material: Material,
    logo_texture: Texture2D
}

pub async fn initialise_background() -> BackgroundData {
    let logo_texture: Texture2D = Texture2D::from_file_with_format(PH_LOGO_TILABLE, None);

    let background_material = load_material(
        ShaderSource::Glsl {
            vertex: BACKGROUND_VERTEX_SHADER,
            fragment: BACKGROUND_FRAGMENT_SHADER,
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
                "logo_texture".to_string()
            ],
            ..Default::default()
        },
    )
    .unwrap();

    return BackgroundData {
        material: background_material,
        logo_texture: logo_texture
    };
}

pub fn draw_background(background_data: &BackgroundData) {
    background_data.material.set_uniform("time", (get_time() % 6.0) as f32);
    background_data.material.set_texture("logo_texture", background_data.logo_texture.to_owned());

    gl_use_material(&background_data.material);
    draw_rectangle(0.0, 0.0, 720.0, 720.0, WHITE);
    gl_use_default_material();
}

const BACKGROUND_VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying vec2 pixel_coord;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    pixel_coord = texcoord * vec2(720.0, 720.0);

    gl_Position = Projection * Model * vec4(position, 1);
}
";

const BACKGROUND_FRAGMENT_SHADER: &'static str = "#version 100
precision lowp float;

varying vec2 pixel_coord;

uniform float time;
uniform sampler2D logo_texture;

void main() {
    gl_FragColor = texture2D(logo_texture, mod(((mod(pixel_coord, 100.0)) / 100.0) + (vec2(1, -0.5) * (mod(time, 6.0) / 3.0)), vec2(1.0, 1.0)));
}
";