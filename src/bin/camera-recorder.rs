use std::{fs, time::Duration};

use libcamera::{
    camera::CameraConfigurationStatus,
    camera_manager::CameraManager,
    framebuffer::AsFrameBuffer,
    framebuffer_allocator::{FrameBuffer, FrameBufferAllocator},
    framebuffer_map::MemoryMappedFrameBuffer,
    pixel_format::PixelFormat,
    request::ReuseFlag,
    stream::StreamRole,
};

const PIXEL_FORMAT_MJPEG: PixelFormat = PixelFormat::new(u32::from_le_bytes(*b"MJPG"), 0);

const FRAME_COUNT: usize = 60;

fn main() {
    // 1. Discover camera
    let manager = CameraManager::new().unwrap();
    let cameras = manager.cameras();
    let cam = cameras.get(0).expect("No camera found");

    // 2. Acquire camera
    let mut cam = cam.acquire().unwrap();

    // 3. Generate configuration
    let mut config = cam
        .generate_configuration(&[StreamRole::ViewFinder])
        .unwrap();

    config
        .get_mut(0)
        .unwrap()
        .set_pixel_format(PIXEL_FORMAT_MJPEG);

    config
        .get_mut(0)
        .unwrap()
        .set_size(libcamera::geometry::Size::new(640, 480));

    // 4. Validate configuration
    match config.validate() {
        CameraConfigurationStatus::Valid => {
            println!("Configuration valid");
        }
        CameraConfigurationStatus::Adjusted => {
            println!("Configuration adjusted: {config:#?}");
        }
        CameraConfigurationStatus::Invalid => {
            panic!("Invalid configuration");
        }
    }

    assert_eq!(
        config.get(0).unwrap().get_pixel_format(),
        PIXEL_FORMAT_MJPEG
    );

    // 5. Configure camera
    cam.configure(&mut config).unwrap();

    // 6. Allocate FrameBuffers
    let mut allocator = FrameBufferAllocator::new(&cam);

    let stream = config.get(0).unwrap().stream().unwrap();
    let buffers = allocator.alloc(&stream).unwrap();

    println!("Allocated {} buffers", buffers.len());

    // 7. Memory-map FrameBuffers
    let buffers = buffers
        .into_iter()
        .map(|buf| MemoryMappedFrameBuffer::new(buf).unwrap())
        .collect::<Vec<_>>();

    // 8. Create Requests
    let requests = buffers
        .into_iter()
        .enumerate()
        .map(|(i, buffer)| {
            let mut request = cam.create_request(Some(i as u64)).unwrap();
            request.add_buffer(&stream, buffer).unwrap();
            request
        })
        .collect::<Vec<_>>();

    // 9. Setup completion channel
    let (tx, rx) = std::sync::mpsc::channel();

    cam.on_request_completed(move |request| {
        tx.send(request).unwrap();
    });

    // 10. Start camera
    cam.start(None).unwrap();

    // 11. Queue all Requests
    for request in requests {
        cam.queue_request(request)
            .map_err(|(_, error)| error)
            .unwrap();
    }

    fs::create_dir_all("frames").unwrap();

    println!("Recording {FRAME_COUNT} frames...");

    // 12. Process completed Requests
    for index in 0..FRAME_COUNT {
        let mut request = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("Camera request timeout");

        // Read frame data
        let jpeg_len: usize;

        {
            let framebuffer: &MemoryMappedFrameBuffer<FrameBuffer> =
                request.buffer(&stream).unwrap();

            let planes = framebuffer.data();
            let jpeg_data = planes.first().unwrap();

            let metadata = framebuffer.metadata().unwrap();

            let jpeg_len = metadata.planes().get(0).unwrap().bytes_used as usize;

            let filename = format!("frames/frame_{index:03}.jpg");

            fs::write(&filename, &jpeg_data[..jpeg_len]).unwrap();

            println!("Frame {index:03}: {jpeg_len} bytes -> {filename}");
        }

        // 13. Reuse Request and its FrameBuffer
        if index + 1 < FRAME_COUNT {
            request.reuse(ReuseFlag::REUSE_BUFFERS);

            cam.queue_request(request)
                .map_err(|(_, error)| error)
                .unwrap();
        }
    }

    // 14. Stop camera
    cam.stop().unwrap();

    println!("Recording completed!");
}
