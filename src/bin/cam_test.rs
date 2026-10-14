use opencv::imgcodecs;

use hackathon::{CarDetector, request_camera_1, request_camera_2};

fn main() {
    let files = [
        "images/car_topdown1.jpeg",
        "images/car_topdown2.jpeg",
        "images/car_topdown3.jpeg",
        "images/car_topdown4.jpeg",
    ];
    // let image = imgcodecs::imread("cam1.jpg", imgcodecs::IMREAD_COLOR).unwrap();
    // let image =
    //     imgcodecs::imdecode(&request_camera_2().as_slice(), imgcodecs::IMREAD_COLOR).unwrap();

    let mut detector = CarDetector::default();
    for file in files {
        let image = imgcodecs::imread(file, imgcodecs::IMREAD_COLOR).unwrap();
        detector.detect(&image);
        for (item, id) in detector.corners.iter().zip(&detector.ids) {
            println!("{id}");
            for item_corner in item {
                println!("    ({}, {})", item_corner.x, item_corner.y)
            }
        }
    }
}
