use reqwest::blocking::ClientBuilder;

pub mod data;

pub fn request_camera(endpoint: &'static str, token: &'static str) -> Vec<u8> {
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

    let bytes = response.bytes().expect("bytes");
    return std::vec::Vec::from(bytes);
}

pub fn request_camera_1() -> Vec<u8> {
    return request_camera(data::CAM_1_ENDPOINT, data::CAM_1_TOKEN);
}

pub fn request_camera_2() -> Vec<u8> {
    return request_camera(data::CAM_2_ENDPOINT, data::CAM_2_TOKEN);
}
