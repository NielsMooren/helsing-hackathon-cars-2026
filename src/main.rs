use std::fs;

use reqwest::blocking::ClientBuilder;

use crate::data::CAM_1_ENDPOINT;

mod data;

// #[tokio::main]
fn main() {
    let mut headers = http::HeaderMap::with_capacity(1);
    headers.insert(
        http::header::AUTHORIZATION,
        http::header::HeaderValue::from_static(data::CAM_1_TOKEN),
    );
    let cam1_endpoint = format!("http://{}/frame", CAM_1_ENDPOINT);
    let mut client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("builder");
    println!("{}", cam1_endpoint);
    let mut response = client.get(&cam1_endpoint).send().expect("response");

    fs::write("cam1.jpg", &response.bytes().unwrap()).expect("write");

    println!("Hello, world!");
}
