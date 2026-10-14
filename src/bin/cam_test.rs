use opencv::imgcodecs;

use hackathon::{CarDetector, request_camera_1, request_camera_2};

fn main() {
    let image = imgcodecs::imread("cam1.jpg", imgcodecs::IMREAD_COLOR).unwrap();
    // let image =
    //     imgcodecs::imdecode(&request_camera_2().as_slice(), imgcodecs::IMREAD_COLOR).unwrap();

    let mut detector = CarDetector::default();
    detector.detect(&image);
    for (item, id) in detector.corners.iter().zip(detector.ids) {
        println!("{id}");
        for item_corner in item {
            println!("    ({}, {})", item_corner.x, item_corner.y)
        }
    }
}
