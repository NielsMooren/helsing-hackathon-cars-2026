use opencv::imgcodecs;

use hackathon::Game;

fn main() {
    let files = [
        "images/car_topdown1.jpeg",
        "images/car_topdown2.jpeg",
        "images/car_topdown3.jpeg",
        "images/car_topdown4.jpeg",
    ];
    // let image =
    //     imgcodecs::imdecode(&request_camera_2().as_slice(), imgcodecs::IMREAD_COLOR).unwrap();

    let mut game = Game::new(2);
    for file in files {
        let image = imgcodecs::imread(file, imgcodecs::IMREAD_COLOR).unwrap();
        let car = game.update(&image).unwrap();
        println!("{file}");
        for (id, marker) in &game.markers {
            println!("  marker {id}: {marker:?}");
        }
        println!("  corners: {:?}", game.arena_corners);
        println!("  car: {car:?}");
    }
}
