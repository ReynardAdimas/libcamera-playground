# Get to know libcamera-rs

- Milestone 1 - Camera Discovery
  - [x] Camera detected
  - [x] Choosing camera by id
  - [x] Show properties and controls
  - [x] Show available stream format
  - [x] Understanding CameraManager lifecycle

- Milestone 2 - Camera Configuration
  - [ ] Able to set resolution
  - [ ] Able to set pixel format
  - [ ] Understanding StreamRole
  - [ ] Understanding validation config process
  - [ ] Understanding relation between Camera and StreamConfiguration

- Milestone 3 - Frame Capture
  - [ ] Capture frame
  - [ ] Able to continuous capture
  - [ ] Understanding Camera Request
  - [ ] Understanding Request completion
  - [ ] Able to count FPS capture

- Milestone 4 - FrameBuffer Management
  - [ ] Understanding mmap
  - [ ] Understanding ownership and lifetime buffer
  - [ ] Accessing raw frame
  - [ ] Understanding plane and stride
  - [ ] Understanding buffer recycling
  - [ ] What is use-after-free / data corrupt

- Milestone 5 - Video Streaming
  - [ ] Raw frame into Gstreamer
  - [ ] Able to encode H264
  - [ ] Saved video into file
  - [ ] Sending video through UDP
  - [ ] Stream using VLC

- Milestone 6 - Concurrent Streaming & Optimization
  - [ ] Capture and encoding not unnecessarily block each other
  - [ ] Stable memory usage
  - [ ] No buffer corruption
  - [ ] Stable FPS
  - [ ] Measured Latency
  - [ ] able to graceful shutdown
