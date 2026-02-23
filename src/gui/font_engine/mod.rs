use macroquad::{
    color::Color,
    text::{Font, TextDimensions, TextParams, draw_text_ex, measure_text},
};

#[derive(Clone, Copy)]
pub struct Point {
    x: f32,
    y: f32,
}

impl Point {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }
}

// if it comes to it, we can make a more advanced font rendering engine but this will do for now
#[must_use]
pub fn draw_text(
    text: &str,
    point: Point,
    width: f32,
    colour: Color,
    font: &Font,
    font_size: u16,
    line_height: f32,
) -> TextDimensions {
    let Point { x, y } = point;
    let space_dimensions = measure_text(" ", Some(font), font_size, 1.0);

    let line_height: f32 = (0.125 * f32::from(font_size)) + (line_height * 10.0);

    let mut current_text_width: f32 = 0.0;
    let mut current_text_y: f32 = space_dimensions.height;
    let mut current_text_properties_setup = false;

    let mut dimensions: TextDimensions = TextDimensions {
        width: 0.0,
        height: 0.0,
        offset_y: 0.0,
    };

    for part in text.split_whitespace() {
        dimensions = measure_text(part, Some(font), font_size, 1.0);

        if !current_text_properties_setup {
            current_text_y = dimensions.offset_y + line_height * 0.6;
            current_text_properties_setup = true;
        }

        if (current_text_width + dimensions.width) > width {
            if current_text_width <= 0.0 {
                draw_text_ex(
                    part,
                    x + current_text_width,
                    y + current_text_y,
                    TextParams {
                        font_size,
                        color: colour,
                        font: Some(font),
                        font_scale: 1.0,
                        ..Default::default()
                    },
                );

                current_text_width = 0.0;
                current_text_y += dimensions.offset_y + line_height;
            } else {
                current_text_width = 0.0;
                current_text_y += dimensions.offset_y + line_height;

                draw_text_ex(
                    part,
                    x + current_text_width,
                    y + current_text_y,
                    TextParams {
                        font_size,
                        color: colour,
                        font: Some(font),
                        font_scale: 1.0,
                        ..Default::default()
                    },
                );

                current_text_width = dimensions.width + space_dimensions.width;
            }
        } else {
            draw_text_ex(
                part,
                x + current_text_width,
                y + current_text_y,
                TextParams {
                    font_size,
                    color: colour,
                    font: Some(font),
                    font_scale: 1.0,
                    ..Default::default()
                },
            );

            current_text_width += dimensions.width + space_dimensions.width;
        }
    }

    current_text_y += line_height * 0.4;

    // I need this for font box visualization, please don't remove
    //macroquad::shapes::draw_rectangle(x, y, width, current_text_y, Color::from_rgba(255, 0, 0, 50));

    TextDimensions {
        width,
        height: current_text_y,
        offset_y: dimensions.offset_y,
    }
}
