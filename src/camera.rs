use opencv::{
    core::{Point2f, Vector},
    objdetect::{
        self, ArucoDetector, DetectorParameters, RefineParameters, get_predefined_dictionary,
    },
    prelude::*,
};

use crate::data;
use reqwest::blocking::ClientBuilder;

pub struct CarDetector {
    detector: ArucoDetector,
    pub corners: Vector<Vector<Point2f>>,
    pub ids: Vector<i32>,
    rejected: Vector<Vector<Point2f>>,
}

impl Default for CarDetector {
    fn default() -> Self {
        let dictionary_type = objdetect::PredefinedDictionaryType::DICT_4X4_50;
        let dictionary =
            get_predefined_dictionary(dictionary_type).expect("could not get aruco dictionary");
        let detector_params =
            DetectorParameters::default().expect("default detector params should work");
        let refine_parameters =
            RefineParameters::new(10.0, 3.0, true).expect("refine params should work");
        let detector = ArucoDetector::new(&dictionary, &detector_params, refine_parameters)
            .expect("should be able to make detector");
        let corners = Vector::<Vector<Point2f>>::new();
        let ids = Vector::<i32>::new();
        let rejected = Vector::<Vector<Point2f>>::new();
        Self {
            detector,
            corners,
            ids,
            rejected,
        }
    }
}
impl CarDetector {
    pub fn detect(&mut self, image: &Mat) {
        self.detector
            .detect_markers(&image, &mut self.corners, &mut self.ids, &mut self.rejected)
            .expect("detection should work");
    }
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
