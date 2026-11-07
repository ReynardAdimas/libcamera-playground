fn main() {
    let width: u64 = 640;
    let height: u64 = 480;
    let fps: u64 = 30;
    let bytes_per_pixel: u64 = 2;

    let frame_size = width * height * bytes_per_pixel;
    let bandwidth_bps = frame_size * fps * 8;

    println!("Resolution: {}x{}", width, height);
    println!("FPS: {}", fps);
    println!("Frame size: {} bytes", frame_size);
    println!("Bandwidth: {:.2} Mbps", bandwidth_bps as f64 / 1_000_000.0);
}
