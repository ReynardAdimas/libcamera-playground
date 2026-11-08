use std::time::Duration;

use libcamera::{
    camera::CameraConfigurationStatus,
    camera_manager::CameraManager,
    framebuffer::AsFrameBuffer,
    framebuffer_allocator::{FrameBuffer, FrameBufferAllocator},
    framebuffer_map::MemoryMappedFrameBuffer,
    pixel_format::PixelFormat,
    stream::StreamRole,
};

// MJPEG FourCC = "MJPG"
const PIXEL_FORMAT_MJPEG: PixelFormat = PixelFormat::new(u32::from_le_bytes(*b"MJPG"), 0);

fn main() {
    // Camera discovery
    let manager = CameraManager::new().unwrap();
    let cameras = manager.cameras();
    let cam = cameras.get(0).expect("No camera found");

    // Acquire camera
    let mut cam = cam.acquire().expect("Unable to acquire camera");

    // Generate configuration
    let mut config = cam
        .generate_configuration(&[StreamRole::ViewFinder])
        .unwrap();

    // Select MJPEG & 640x480
    config
        .get_mut(0)
        .unwrap()
        .set_pixel_format(PIXEL_FORMAT_MJPEG);

    config
        .get_mut(0)
        .unwrap()
        .set_size(libcamera::geometry::Size::new(640, 480));

    // Validate
    match config.validate() {
        CameraConfigurationStatus::Valid => println!("Configuration valid"),
        CameraConfigurationStatus::Adjusted => println!("Configuration adjusted: {config:#?}"),
        CameraConfigurationStatus::Invalid => panic!("Invalid camera configuration"),
    }

    // Ensure MJPEG was retained
    assert_eq!(
        config.get(0).unwrap().get_pixel_format(),
        PIXEL_FORMAT_MJPEG
    );

    // Apply configuration
    cam.configure(&mut config).unwrap();

    // Allocate FrameBuffers
    let mut allocator = FrameBufferAllocator::new(&cam);

    let stream = config.get(0).unwrap().stream().unwrap();
    let buffers = allocator.alloc(&stream).unwrap();

    println!("Allocated {} buffers", buffers.len());

    // Memory-map buffers
    let buffers = buffers
        .into_iter()
        .map(|buf| MemoryMappedFrameBuffer::new(buf).unwrap())
        .collect::<Vec<_>>();

    // Create Request and attach buffers
    let mut requests = buffers
        .into_iter()
        .map(|buffer| {
            let mut request = cam.create_request(None).unwrap();
            request.add_buffer(&stream, buffer).unwrap();
            request
        })
        .collect::<Vec<_>>();

    // Receive completed request
    let (tx, rx) = std::sync::mpsc::channel();

    cam.on_request_completed(move |request| {
        tx.send(request).unwrap();
    });

    // Start camera
    cam.start(None).unwrap();

    // Queue one request
    cam.queue_request(requests.pop().unwrap())
        .map_err(|(_, error)| error)
        .unwrap();

    println!("Waiting for camera");

    let request = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("Camera request timeout");

    println!("Capture completed");

    // Read FrameBuffer
    let framebuffer: &MemoryMappedFrameBuffer<FrameBuffer> = request.buffer(&stream).unwrap();

    let planes = framebuffer.data();
    let jpeg_data = planes.first().unwrap();

    // Get actual JPEG data length
    let jpeg_len = framebuffer
        .metadata()
        .unwrap()
        .planes()
        .get(0)
        .unwrap()
        .bytes_used as usize;

    // Save JPEG
    std::fs::write("output.jpg", &jpeg_data[..jpeg_len]).unwrap();

    println!("Saved output.jpg ({jpeg_len} bytes)");
}
