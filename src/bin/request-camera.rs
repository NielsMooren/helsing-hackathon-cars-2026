use std::fs;

use hackathon::{request_camera_2, request_camera_1};

// #[tokio::main]
fn main() {
    fs::write("cam1.jpg", request_camera_1()).expect("write");
    fs::write("cam2.jpg", request_camera_2()).expect("write");
    println!("Hello, world!");
}
