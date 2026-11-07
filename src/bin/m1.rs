use libcamera::{camera_manager::CameraManager, stream::StreamRole};

fn main() {
    let manager = CameraManager::new().unwrap();
    let cameras = manager.cameras();
    println!("Total cameras: {}", cameras.len());

    for (index, camera) in cameras.iter().enumerate() {
        println!("Camera [{}]: {}", index, camera.id());
    }

    // more detail camera info
    for (index, camera) in cameras.iter().enumerate() {
        println!("\nCamera [{}]", index);
        println!("ID: {}", camera.id());

        println!("Properties:");
        println!("{:#?}", camera.properties());

        println!("Controls:");
        println!("{:#?}", camera.controls());
    }

    let camera = cameras.get(0).unwrap();
    let config = camera
        .generate_configuration(&[StreamRole::ViewFinder])
        .unwrap();

    println!("\nGenerated configuration:");
    println!("{:#?}", config);

    let view_finder_cfg = config.get(0).unwrap();

    println!("Available formats: {:#?}", view_finder_cfg.formats());

    // frame size
    let width = 640;
    let height = 480;
    let bytes_per_pixel = 2;

    let frame_size = width * height * bytes_per_pixel;

    println!("Frame size: {} bytes", frame_size);
    println!("Frame size: {:.2} KiB", frame_size as f64 / 1024.0);
}
