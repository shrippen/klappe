import io.github.shrippen.klappe

// The Rust element that draws the camera's image; its own file, so the QML tests, which do
// not know Rust types, can load `CameraView` with a stand-in (`surfaceSource`).
CameraPicture {}
