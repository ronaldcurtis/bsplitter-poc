mod utils;
use image::DynamicImage;
use image::GrayImage;
use image::Rgba;
use image::RgbaImage;
use rustface::Detector;
use rustface::FaceInfo;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{console::log_1, CanvasRenderingContext2d, ImageData};
use rustface::{ImageData as FaceImageData, read_model};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;

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

pub static FACE_DETECTOR_MODEL: &[u8] = include_bytes!("../assets/seeta_fd_frontal_v1.0.bin");

#[wasm_bindgen]
pub fn process_image(ctx: &CanvasRenderingContext2d, source_image_data: &mut [u8], width: u32, height: u32) -> Result<(), JsValue> {
    let rgba = RgbaImage::from_raw(width, height, source_image_data.to_vec()).unwrap_throw();
    let dynamic_image =  DynamicImage::from(rgba);


    let model = read_model(FACE_DETECTOR_MODEL).unwrap_throw();
    let mut detector = rustface::create_detector_with_model(model);
    detector.set_min_face_size(20);
    detector.set_score_thresh(2.0);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);

    let mut rgba = dynamic_image.to_rgba8();
    let faces = detect_faces(&mut *detector, &dynamic_image.to_luma8());
    for face in faces {
        let bbox = face.bbox();
        let rect = Rect::at(bbox.x(), bbox.y()).of_size(bbox.width(), bbox.height());

        draw_hollow_rect_mut(&mut rgba, rect, Rgba([255, 0, 0, 255]));
    }

    let new_image_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), width, height)?;
    ctx.put_image_data(&new_image_data, 0.0, 0.0)
}

fn detect_faces(detector: &mut dyn Detector, gray: &GrayImage) -> Vec<FaceInfo> {
    let (width, height) = gray.dimensions();
    let mut image = FaceImageData::new(gray, width, height);
    let faces = detector.detect(&mut image);
    faces
}
