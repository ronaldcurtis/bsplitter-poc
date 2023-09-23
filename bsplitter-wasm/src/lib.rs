mod utils;
use dcv_color_primitives::{convert_image, ColorSpace, ErrorKind, ImageFormat, PixelFormat};
use imageproc::drawing::{draw_cross_mut, draw_hollow_rect_mut};
use pico_detect::{
    image::{DynamicImage, GrayImage, RgbImage, Rgba, RgbaImage},
    nalgebra::{center, Isometry2, Point2, Similarity2},
    Detection, Detector, Localizer, MultiScale, Rect as PicoRect, Shaper,
};
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{console::log_1, CanvasRenderingContext2d, ImageData};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! console_log {
    ( $( $t:tt )* ) => {
        log_1(&format!( $( $t )* ).into());
    }
}

const SRC_FORMAT: ImageFormat = ImageFormat {
    pixel_format: PixelFormat::Nv12,
    color_space: ColorSpace::Bt709,
    num_planes: 1,
};

const DST_FORMAT: ImageFormat = ImageFormat {
    pixel_format: PixelFormat::Rgb,
    color_space: ColorSpace::Rgb,
    num_planes: 1,
};

#[wasm_bindgen]
extern "C" {}

#[wasm_bindgen]
pub fn greet() {
    console_log!("Hello! From wasm!");
}

#[wasm_bindgen]
pub fn process_video_frame(
    ctx: &CanvasRenderingContext2d,
    raw_video_pixels: &mut [u8],
    width: u32,
    height: u32,
) -> Result<(), JsValue> {
    dcv_color_primitives::initialize();
    let length: u32 = 3 * width * height;
    let mut dst_data = vec![0; length as usize];

    let _ = convert_image(
        width,
        height,
        &SRC_FORMAT,
        None,
        &[raw_video_pixels],
        &DST_FORMAT,
        None,
        &mut [&mut dst_data],
    )
    .map_err(|e| match e {
        ErrorKind::NotEnoughData => JsError::new("convert_image:NotEnoughData"),
        ErrorKind::InvalidOperation => JsError::new("convert_image:InvalidOperation"),
        ErrorKind::NotInitialized => JsError::new("convert_image:NotInitialized"),
        ErrorKind::InvalidValue => JsError::new("convert_image:InvalidValue"),
    })?;

    let rgb_image_buffer = RgbImage::from_raw(width, height, dst_data).unwrap_throw();
    let dyn_image = DynamicImage::from(DynamicImage::ImageRgb8(rgb_image_buffer));
    let (gray_image_buffer, mut rgba_image_buffer) = (dyn_image.to_luma8(), dyn_image.to_rgba8());
    let (facefinder, mut shaper, puploc) = load_models();
    let faces = detect_faces_and_landmarks(&gray_image_buffer, &facefinder, &mut shaper, &puploc);
    for face in faces.iter() {
        draw_face(&mut rgba_image_buffer, &face);
    }

    let new_image_data =
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba_image_buffer), width, height)?;
    ctx.put_image_data(&new_image_data, 0.0, 0.0)
}

pub static FACE_DETECTOR_MODEL: &[u8] = include_bytes!("../assets/pico-face-detector.bin");
pub static LOCALIZER_MODEL: &[u8] = include_bytes!("../assets/puploc.bin");
pub static SHAPER_MODEL: &[u8] = include_bytes!("../assets/shaper_5_face_landmarks.bin");

const THRESHOLD: f32 = 0.2;
const SHIFT_FACTOR: f32 = 0.05;
const SCALE_FACTOR: f32 = 1.1;

struct Face {
    rect: PicoRect,
    shape: Vec<Point2<f32>>,
    pupils: (Point2<f32>, Point2<f32>),
}

fn load_models() -> (Detector, Shaper, Localizer) {
    let facefinder = Detector::from_readable(FACE_DETECTOR_MODEL).unwrap();
    let puploc = Localizer::from_readable(LOCALIZER_MODEL).unwrap();
    let shaper = Shaper::from_readable(SHAPER_MODEL).unwrap();

    (facefinder, shaper, puploc)
}

fn detect_faces_and_landmarks(
    gray: &GrayImage,
    detector: &Detector,
    shaper: &mut Shaper,
    localizer: &Localizer,
) -> Vec<Face> {
    // initialize multiscale
    let multiscale = MultiScale::default()
        .with_size_range(100, gray.width())
        .with_shift_factor(SHIFT_FACTOR)
        .with_scale_factor(SCALE_FACTOR);

    // source of "randomness" for perturbated search for pupil
    let mut rng = XorShiftRng::seed_from_u64(42u64);
    let nperturbs = 31usize;

    Detection::clusterize(multiscale.run(detector, gray).as_mut(), THRESHOLD)
        .iter()
        .filter_map(|detection| {
            if detection.score() < 40.0 {
                return None;
            }

            let (center, size) = (detection.center(), detection.size());
            let rect = PicoRect::at(
                (center.x - size / 2.0) as i32,
                (center.y - size / 2.0) as i32,
            )
            .of_size(size as u32, size as u32);

            let shape = shaper.predict(gray, rect);
            let pupils = Shape5::find_eyes_roi(&shape);
            let pupils = (
                localizer.perturb_localize(gray, pupils.0, &mut rng, nperturbs),
                localizer.perturb_localize(gray, pupils.1, &mut rng, nperturbs),
            );

            Some(Face {
                rect,
                shape,
                pupils,
            })
        })
        .collect::<Vec<Face>>()
}

fn draw_face(image: &mut RgbaImage, face: &Face) {
    draw_hollow_rect_mut(image, face.rect, Rgba([0, 0, 255, 255]));

    for (_i, point) in face.shape.iter().enumerate() {
        draw_cross_mut(
            image,
            Rgba([0, 255, 0, 255]),
            point.x as i32,
            point.y as i32,
        );
    }

    draw_cross_mut(
        image,
        Rgba([255, 0, 0, 255]),
        face.pupils.0.x as i32,
        face.pupils.0.y as i32,
    );
    draw_cross_mut(
        image,
        Rgba([255, 0, 0, 255]),
        face.pupils.1.x as i32,
        face.pupils.1.y as i32,
    );
}

enum Shape5 {
    LeftOuterEyeCorner = 0,
    LeftInnerEyeCorner = 1,
    RightOuterEyeCorner = 2,
    RightInnerEyeCorner = 3,
}

impl Shape5 {
    fn find_eyes_roi(shape: &[Point2<f32>]) -> (Similarity2<f32>, Similarity2<f32>) {
        let (li, lo) = (
            &shape[Self::LeftInnerEyeCorner as usize],
            &shape[Self::LeftOuterEyeCorner as usize],
        );
        let (ri, ro) = (
            &shape[Self::RightInnerEyeCorner as usize],
            &shape[Self::RightOuterEyeCorner as usize],
        );

        let (dl, dr) = (lo - li, ri - ro);
        let (l, r) = (li + dl.scale(0.5), ro + dr.scale(0.5));

        (
            Similarity2::from_isometry(Isometry2::translation(l.x, l.y), dl.norm() * 1.1),
            Similarity2::from_isometry(Isometry2::translation(r.x, r.y), dr.norm() * 1.1),
        )
    }
}
