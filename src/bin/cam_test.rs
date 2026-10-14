use opencv::{
    core::{Point2f, Vector},
    imgcodecs,
    objdetect::{
        self, ArucoDetector, DetectorParameters, RefineParameters, get_predefined_dictionary,
    },
    prelude::*,
};

use hackathon::{request_camera_1, request_camera_2};

fn main() {
    // let image = imgcodecs::imread("cam1.jpg", imgcodecs::IMREAD_COLOR).unwrap();
    let image =
        imgcodecs::imdecode(&request_camera_2().as_slice(), imgcodecs::IMREAD_COLOR).unwrap();
    let dictionary_type = objdetect::PredefinedDictionaryType::DICT_4X4_50;
    let dictionary =
        get_predefined_dictionary(dictionary_type).expect("could not get aruco dictionary");
    let detector_params =
        DetectorParameters::default().expect("default detector params should work");
    let refine_parameters =
        RefineParameters::new(10.0, 3.0, true).expect("refine params should work");
    let detector = ArucoDetector::new(&dictionary, &detector_params, refine_parameters)
        .expect("should be able to make detector");
    let mut corners = Vector::<Vector<Point2f>>::new();
    let mut ids = Vector::<i32>::new();
    let mut rejected_ids = Vector::<Vector<Point2f>>::new();
    detector
        .detect_markers(&image, &mut corners, &mut ids, &mut rejected_ids)
        .expect("detection should work");

    for (item, id) in corners.iter().zip(ids) {
        println!("{id}");
        for item_corner in item {
            println!("    ({}, {})", item_corner.x, item_corner.y)
        }
    }
}
