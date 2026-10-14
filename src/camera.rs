use std::collections::HashMap;

use opencv::{
    core::{self, Point2f, Vector},
    geometry,
    objdetect::{
        self, ArucoDetector, DetectorParameters, RefineParameters, get_predefined_dictionary,
    },
    prelude::*,
};

use crate::data;
use reqwest::blocking::ClientBuilder;

/// Marker ids for the arena's top-left, top-right, bottom-right, bottom-left corners.
pub const CORNER_IDS: [i32; 4] = [0, 12, 13, 14];
/// Arena map coordinates for top-left, top-right, bottom-right, bottom-left (0..1).
const MAP: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

/// The four image-space corners of a marker, clockwise from its top-left.
pub type Marker = [Point2f; 4];

pub struct Detector {
    detector: ArucoDetector,
    corners: Vector<Vector<Point2f>>,
    ids: Vector<i32>,
    rejected: Vector<Vector<Point2f>>,
}

impl Default for Detector {
    fn default() -> Self {
        let dictionary_type = objdetect::PredefinedDictionaryType::DICT_4X4_50;
        let dictionary =
            get_predefined_dictionary(dictionary_type).expect("could not get aruco dictionary");
        let mut detector_params =
            DetectorParameters::default().expect("default detector params should work");
        detector_params.set_adaptive_thresh_win_size_min(3);
        detector_params.set_adaptive_thresh_win_size_max(53);
        detector_params.set_adaptive_thresh_win_size_step(5);
        detector_params.set_min_marker_perimeter_rate(0.01);
        detector_params.set_error_correction_rate(1.0);
        detector_params.set_corner_refinement_method(
            objdetect::CornerRefineMethod::CORNER_REFINE_SUBPIX as i32,
        );
        let refine_parameters = RefineParameters::new_def().expect("refine params should work");
        let detector = ArucoDetector::new(&dictionary, &detector_params, refine_parameters)
            .expect("should be able to make detector");
        Self {
            detector,
            corners: Vector::new(),
            ids: Vector::new(),
            rejected: Vector::new(),
        }
    }
}

impl Detector {
    /// Every marker visible in `image`, keyed by id.
    pub fn detect(&mut self, image: &Mat) -> opencv::Result<HashMap<i32, Marker>> {
        self.detector
            .detect_markers(image, &mut self.corners, &mut self.ids, &mut self.rejected)?;
        self.ids
            .iter()
            .zip(self.corners.iter())
            .map(|(id, c)| Ok((id, [c.get(0)?, c.get(1)?, c.get(2)?, c.get(3)?])))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Car {
    /// Position in arena coordinates (0..1).
    pub pos: Point2f,
    /// Heading in radians in arena coordinates, measured from +x towards +y.
    pub dir: f32,
}

pub struct Game {
    pub car_id: i32,
    /// Last seen centre of each arena corner, in `CORNER_IDS` order.
    pub arena_corners: [Option<Point2f>; 4],
    /// Homography from image pixels to the arena map, once all corners have been seen.
    pub arena_t: Option<Mat>,
    /// All markers from the latest frame.
    pub markers: HashMap<i32, Marker>,
    /// Last seen marker of the car, in image pixels.
    pub car_marker: Option<Marker>,
    /// Last known car pose.
    pub car: Option<Car>,
    detector: Detector,
}

impl Game {
    pub fn new(car_id: i32) -> Self {
        Self {
            car_id,
            arena_corners: [None; 4],
            arena_t: None,
            markers: HashMap::new(),
            car_marker: None,
            car: None,
            detector: Detector::default(),
        }
    }

    /// Detects markers in `image`, refreshes the arena transform and car pose, and returns the
    /// latest car pose (the previous one if the car isn't visible in this frame).
    pub fn update(&mut self, image: &Mat) -> opencv::Result<Option<Car>> {
        self.markers = self.detector.detect(image)?;

        let mut corners_moved = false;
        for (slot, id) in self.arena_corners.iter_mut().zip(CORNER_IDS) {
            if let Some(marker) = self.markers.get(&id) {
                *slot = Some(marker_center(marker));
                corners_moved = true;
            }
        }
        if corners_moved && let Some(t) = self.arena_transform()? {
            self.arena_t = Some(t);
        }

        if let Some(&marker) = self.markers.get(&self.car_id) {
            self.car_marker = Some(marker);
            if let Some(car) = self.car_pose(&marker)? {
                self.car = Some(car);
            }
        }
        Ok(self.car)
    }

    /// Homography from image pixels to the arena map, or `None` until every corner has been seen.
    pub fn arena_transform(&self) -> opencv::Result<Option<Mat>> {
        let [Some(tl), Some(tr), Some(br), Some(bl)] = self.arena_corners else {
            return Ok(None);
        };
        let dst: Vec<Point2f> = MAP.iter().map(|&(x, y)| Point2f::new(x, y)).collect();
        geometry::get_perspective_transform(
            &Vector::from_slice(&[tl, tr, br, bl]),
            &Vector::from_slice(&dst),
            core::DECOMP_LU,
        )
        .map(Some)
    }

    /// Pose of `marker` in arena coordinates, or `None` without an arena transform.
    pub fn car_pose(&self, marker: &Marker) -> opencv::Result<Option<Car>> {
        let Some(h) = &self.arena_t else {
            return Ok(None);
        };
        let mut pts = Vector::<Point2f>::new();
        core::perspective_transform(&Vector::from_slice(marker), &mut pts, h)?;
        let pts: Marker = [pts.get(0)?, pts.get(1)?, pts.get(2)?, pts.get(3)?];
        let pos = marker_center(&pts);
        let front = mean(&pts[0..2]); // middle of the marker's top edge
        Ok(Some(Car {
            pos,
            dir: (front.y - pos.y).atan2(front.x - pos.x),
        }))
    }
}

/// Intersection of the marker's diagonals, which stays the true centre under perspective.
fn marker_center(p: &Marker) -> Point2f {
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let h = |p: Point2f| [p.x, p.y, 1.0];
    let c = cross(cross(h(p[0]), h(p[2])), cross(h(p[1]), h(p[3])));
    Point2f::new(c[0] / c[2], c[1] / c[2])
}

fn mean(pts: &[Point2f]) -> Point2f {
    let n = pts.len() as f32;
    Point2f::new(
        pts.iter().map(|p| p.x).sum::<f32>() / n,
        pts.iter().map(|p| p.y).sum::<f32>() / n,
    )
}

fn request_camera(endpoint: &'static str, token: &'static str) -> Vec<u8> {
    let mut headers = http::HeaderMap::with_capacity(1);
    headers.insert(
        http::header::AUTHORIZATION,
        http::header::HeaderValue::from_static(token),
    );
    let cam1_endpoint = format!("http://{}/frame", endpoint);
    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("builder");
    // println!("{}", cam1_endpoint);

    let response = client.get(&cam1_endpoint).send().expect("response");

    response.bytes().expect("bytes").to_vec()
}

pub fn request_camera_1() -> Vec<u8> {
    request_camera(data::CAM_1_ENDPOINT, data::CAM_1_TOKEN)
}

pub fn request_camera_2() -> Vec<u8> {
    request_camera(data::CAM_2_ENDPOINT, data::CAM_2_TOKEN)
}

