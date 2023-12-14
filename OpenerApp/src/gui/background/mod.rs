use macroquad::{prelude::*, miniquad::{BlendState, Equation, BlendFactor, BlendValue}};

const PH_LOGO_TILABLE: &[u8] = include_bytes!("./ph-logo-tilable.png");
const BACKGROUND_VERTEX_SHADER: &'static str = include_str!("./vertex_shader.glsl");
const BACKGROUND_FRAGMENT_SHADER: &'static str = include_str!("./fragment_shader.glsl");
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