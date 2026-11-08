use std::time::{Duration, Instant};

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

const FRAME_COUNT: usize = 100;

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

    println!("\nCamera Frame Monitor");
    println!("====================");
    println!("Monitoring {FRAME_COUNT} frames...\n");

    let start = Instant::now();
    let mut window_start = Instant::now();
    let mut window_frames = 0usize;
    let mut total_bytes = 0usize;

    // 12. Monitor completed Requests
    for index in 0..FRAME_COUNT {
        let mut request = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("Camera request timeout");

        let sequence = request.sequence();

        let jpeg_len;

        {
            let framebuffer: &MemoryMappedFrameBuffer<FrameBuffer> =
                request.buffer(&stream).unwrap();

            let metadata = framebuffer.metadata().unwrap();

            jpeg_len = metadata.planes().get(0).unwrap().bytes_used as usize;
        }

        total_bytes += jpeg_len;
        window_frames += 1;

        // Print frame information
        println!(
            "Frame #{:03} | Sequence: {:03} | {} bytes",
            index + 1,
            sequence,
            jpeg_len
        );

        // Calculate FPS every ~1 second
        let window_elapsed = window_start.elapsed();

        if window_elapsed >= Duration::from_secs(1) {
            let fps = window_frames as f64 / window_elapsed.as_secs_f64();

            println!("          --> {:.2} FPS\n", fps);

            window_frames = 0;
            window_start = Instant::now();
        }

        // 13. Reuse Request and FrameBuffer
        if index + 1 < FRAME_COUNT {
            request.reuse(ReuseFlag::REUSE_BUFFERS);

            cam.queue_request(request)
                .map_err(|(_, error)| error)
                .unwrap();
        }
    }

    // 14. Stop camera
    cam.stop().unwrap();

    // 15. Final statistics
    let elapsed = start.elapsed();
    let average_fps = FRAME_COUNT as f64 / elapsed.as_secs_f64();

    println!("\n====================");
    println!("Monitoring completed!");
    println!("Total frames : {FRAME_COUNT}");
    println!("Total bytes  : {total_bytes}");
    println!("Elapsed      : {:.2} seconds", elapsed.as_secs_f64());
    println!("Average FPS  : {:.2}", average_fps);
}
