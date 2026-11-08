use libcamera::{
    camera_manager::CameraManager, geometry::Size, pixel_format::PixelFormat, stream::StreamRole,
};

fn main() {
    // 1. Membuat CameraManager
    let manager = CameraManager::new().expect("Camera manager");

    // 2. Mendapatkan kamera pertama
    let cameras = manager.cameras();

    let Some(camera) = cameras.get(0) else {
        eprintln!("No cameras available");
        return;
    };

    // 3. Acquire kamera
    let mut camera = camera.acquire().expect("Acquire camera");

    // 4. Generate konfigurasi
    let mut config = camera
        .generate_configuration(&[StreamRole::ViewFinder])
        .expect("Generate configuration");

    // 5. Mengubah pixel format dan resolusi
    if let Some(mut stream_config) = config.get_mut(0) {
        let yuyv = PixelFormat::parse("YUYV").expect("YUYV pixel format");

        stream_config.set_pixel_format(yuyv);
        stream_config.set_size(Size::new(1920, 1080));
    }

    // 6. Validate konfigurasi
    let status = config.validate();

    println!("Validation status: {status:?}");

    // 7. Menerapkan konfigurasi
    camera.configure(&mut config).expect("Configure camera");

    // 8. Membaca konfigurasi setelah diterapkan
    if let Some(stream_config) = config.get(0) {
        println!("\nConfigured stream:");
        println!("Pixel format: {:?}", stream_config.get_pixel_format());
        println!("Size: {:?}", stream_config.get_size());
        println!("Stride: {}", stream_config.get_stride());
        println!("Frame size: {}", stream_config.get_frame_size());
        println!("Buffer count: {}", stream_config.get_buffer_count());
    }
}
