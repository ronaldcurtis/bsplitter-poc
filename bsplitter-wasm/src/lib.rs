mod utils;
use imageproc::drawing::{draw_cross_mut, draw_hollow_rect_mut};
use pico_detect::{
    image::{
        DynamicImage::{self, ImageRgba8},
        GrayImage, Rgba, RgbaImage,
    },
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

#[wasm_bindgen]
extern "C" {}

#[wasm_bindgen]
pub fn greet() {
    console_log!("Hello, bsplitter-wasm! From a worker");
}

pub static FACE_DETECTOR_MODEL: &[u8] = include_bytes!("../assets/pico-face-detector.bin");
pub static LOCALIZER_MODEL: &[u8] = include_bytes!("../assets/puploc.bin");
pub static SHAPER_MODEL: &[u8] = include_bytes!("../assets/shaper_5_face_landmarks.bin");

const THRESHOLD: f32 = 0.2;
const SHIFT_FACTOR: f32 = 0.05;
const SCALE_FACTOR: f32 = 1.1;

struct Face {
    score: f32,
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

fn detect_faces(
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
                score: detection.score(),
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
    #[allow(dead_code)]
    Nose = 4,
}

impl Shape5 {
    fn size() -> usize {
        5
    }

    #[allow(dead_code)]
    fn find_eye_centers(shape: &[Point2<f32>]) -> (Point2<f32>, Point2<f32>) {
        assert_eq!(shape.len(), Self::size());
        (
            center(
                &shape[Self::LeftInnerEyeCorner as usize],
                &shape[Self::LeftOuterEyeCorner as usize],
            ),
            center(
                &shape[Self::RightInnerEyeCorner as usize],
                &shape[Self::RightOuterEyeCorner as usize],
            ),
        )
    }

    fn find_eyes_roi(shape: &[Point2<f32>]) -> (Similarity2<f32>, Similarity2<f32>) {
        assert_eq!(shape.len(), Self::size());
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

#[wasm_bindgen]
pub fn process_image(
    ctx: &CanvasRenderingContext2d,
    source_image_data: &mut [u8],
    width: u32,
    height: u32,
) -> Result<(), JsValue> {
    let rgba = RgbaImage::from_raw(width, height, source_image_data.to_vec()).unwrap_throw();
    let dyn_image = DynamicImage::from(ImageRgba8(rgba));
    let (gray, mut image) = (dyn_image.to_luma8(), dyn_image.to_rgba8());
    let (facefinder, mut shaper, puploc) = load_models();
    let faces = detect_faces(&gray, &facefinder, &mut shaper, &puploc);
    for face in faces.iter() {
        draw_face(&mut image, &face);
    }
    let new_image_data =
        ImageData::new_with_u8_clamped_array_and_sh(Clamped(&image), width, height)?;
    ctx.put_image_data(&new_image_data, 0.0, 0.0)
}
