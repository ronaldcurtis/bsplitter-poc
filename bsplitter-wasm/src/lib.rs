mod utils;
use image::RgbaImage;
use image::imageops::colorops::brighten_in_place;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{console::log_1, CanvasRenderingContext2d, ImageData};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! console_log {
    ( $( $t:tt )* ) => {
        log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
extern "C" {}

#[wasm_bindgen]
pub fn greet() {
    console_log!("Hello, bsplitter-wasm! From a worker");
}

#[wasm_bindgen]
pub fn process_image(ctx: &CanvasRenderingContext2d, source_image_data: &mut [u8], width: u32, height: u32) -> Result<(), JsValue> {
    let mut image_buffer = RgbaImage::from_raw(width, height, source_image_data.to_vec()).unwrap_throw();
    brighten_in_place(&mut image_buffer, 50);
    let new_image_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&image_buffer), width, height)?;
    ctx.put_image_data(&new_image_data, 0.0, 0.0)
}
