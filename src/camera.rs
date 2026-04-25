use base64::Engine;
use base64::engine::general_purpose;
use image::ImageFormat;
use nokhwa::{
    Camera,
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};
use std::error::Error;
use std::io::Cursor;

pub fn capture_photo() -> Result<String, Box<dyn Error + Sync + Send>> {
    let index = CameraIndex::Index(0);
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = Camera::new(index, requested)?;

    let frame = camera.frame()?;
    let decoded = frame.decode_image::<RgbFormat>()?;
    let mut buffer = Cursor::new(Vec::new());
    decoded.write_to(&mut buffer, ImageFormat::Avif)?;
    let base64 = general_purpose::STANDARD.encode(buffer.into_inner());

    Ok(format!("data:image/avif;base64,{base64}"))
}
